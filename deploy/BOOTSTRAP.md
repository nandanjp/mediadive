# Cluster bootstrap

One-time setup of the homelab cluster, run by hand. Everything after this arrives
by commit: Argo CD reconciles `deploy/charts/mediadive` from `main`.

**Read [`../CLAUDE.md`](../CLAUDE.md) first** for what mediadive is and where
things are defined. Architecture context: [`../docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md);
deployment model: [`../docs/DELIVERY.md`](../docs/DELIVERY.md).

Scope is cluster-level infrastructure only — Argo CD, Argo Rollouts, operators, a
StorageClass, the tunnel route. **Postgres, Garage, Redis, Meilisearch and the
application itself are not installed here.** They are defined in the Helm chart
and appear when Argo syncs, which is the point of step 9.

## Facts

Fill in the blanks and commit this file.

| Fact | Value |
|---|---|
| Hostname | `mediadive.nandan-hl.dev` |
| Cloudflare zone | `nandan-hl.dev`, Cloudflare-managed |
| Tunnel | Existing — add a public hostname route, do not create a new tunnel |
| Nodes | 1 |
| CPU / RAM | 12 cores / 15.7 GB |
| Data disk | `/mnt/drive2` — Postgres volumes **and** Garage |
| Cluster state at start | **Not empty** — see the note below |
| Node name | `homelab` (192.168.1.157) |
| k3s version | `v1.35.5+k3s1`, containerd `2.2.3-k3s1` |
| cloudflared location | Container — `Deployment/cloudflared` in the `cloudflared` namespace, 2 replicas |
| cloudflared management | Cloudflare dashboard (token mode: `tunnel --no-autoupdate run` with `TUNNEL_TOKEN`) |

**Correction, recorded 2026-09-20.** The cluster was *not* empty at start. It
carries `vault`, `personal` and `cloudflared` namespaces from earlier work on
this box. Every application Deployment and StatefulSet in `vault` and
`personal` is scaled to **0**, so nothing competes for CPU or memory — but the
Services, PVCs and PVs are live objects, and four PVs in `vault` carry a
`deletionTimestamp` and must not be disturbed. Treat those two namespaces as
out of bounds. Headroom at bootstrap: 12 cores, 13 GiB of 14 GiB free.

**Accepted risk:** Postgres and Garage share `/mnt/drive2`, so a drive failure
loses the database and its backups together. Deliberate, and recorded in
`ARCHITECTURE.md`. This is why the restore test in step 10 is mandatory rather
than optional.

## Versions

Chart versions are not pinned in advance — resolve each, then **record what you
installed** so later runs reproduce this one rather than drifting to whatever is
current.

| Component | Chart | Resolve with | Installed |
|---|---|---|---|
| Argo CD | `argo/argo-cd` | `helm search repo argo/argo-cd --versions \| head -3` | `10.9.2` (app v3.5.3) |
| Argo Rollouts | `argo/argo-rollouts` | `helm search repo argo/argo-rollouts --versions \| head -3` | `2.43.2` (app v1.10.0) |
| CloudNativePG | `cnpg/cloudnative-pg` | `helm search repo cnpg/cloudnative-pg --versions \| head -3` | `0.29.0` (app 1.30.0) |
| Prometheus stack | `prometheus-community/kube-prometheus-stack` | `helm search repo prometheus-community/kube-prometheus-stack --versions \| head -3` | `91.4.1` (app v0.94.0) |
| Loki | `grafana/loki` | `helm search repo grafana/loki --versions \| head -3` | `7.3.0` (app 3.6.12) |
| Alloy | `grafana/alloy` | `helm search repo grafana/alloy --versions \| head -3` | `1.12.1` (app v1.19.2) |

## Before starting

- `kubectl`, `helm` and `age` available on the machine.
- Write access to this repository, on a branch — **never push to `main`.**
- A password manager to hold the age private key (step 3).

---

## 1 · Confirm the cluster

```sh
kubectl get nodes -o wide
kubectl -n kube-system get pods -l app.kubernetes.io/name=traefik
kubectl version
```

**Verify:** exactly one node `Ready`; the Traefik pod `Running`. Record the node
name and k3s version in the facts table.

Traefik binds ports 80 and 443 on the node, which is what the tunnel will point
at in step 8.

## 2 · Namespaces

```sh
kubectl create namespace argocd
kubectl create namespace mediadive
kubectl create namespace observability
```

**Verify:** `kubectl get ns argocd mediadive observability`

Re-runnable: `create namespace` fails harmlessly if it already exists.

## 3 · age key — the one secret installed by hand

```sh
age-keygen -o age.key          # prints the public key to stderr
kubectl -n argocd create secret generic sops-age --from-file=keys.txt=age.key
```

**Before deleting the key file, store the private key in a password manager.**
If both the file and the cluster secret are lost, every SOPS-encrypted value in
this repository becomes permanently unreadable and has to be regenerated.

```sh
rm -f age.key
```

Then commit `.sops.yaml` at the repository root with the **public** recipient:

```yaml
creation_rules:
  - path_regex: deploy/.*\.sops\.ya?ml$
    age: age1...            # the public key from age-keygen
```

**Verify:** `kubectl -n argocd get secret sops-age`; `age.key` no longer on disk;
`.sops.yaml` committed. The private key must never appear in git — the repository
is public.

## 4 · Argo CD

```sh
helm repo add argo https://argoproj.github.io/argo-helm
helm repo update
helm install argocd argo/argo-cd -n argocd --version <recorded> -f argocd-values.yaml
```

`argocd-values.yaml` has to teach the repo-server to decrypt SOPS, which Argo CD
cannot do natively. It needs the `sops` binary and the `helm-secrets` plugin on
the repo-server, the age key mounted from the `sops-age` secret, and
`helm.valuesFileSchemes` extended to include `secrets`:

```yaml
repoServer:
  env:
    - name: HELM_PLUGINS
      value: /helm-plugins/helm-secrets/
    - name: SOPS_AGE_KEY_FILE
      value: /helm-secrets-private-keys/keys.txt
  volumes:
    - name: helm-plugins
      emptyDir: {}
    - name: sops-age
      secret:
        secretName: sops-age
  volumeMounts:
    - name: helm-plugins
      mountPath: /helm-plugins
    - name: sops-age
      mountPath: /helm-secrets-private-keys
  initContainers:
    - name: install-sops-and-helm-secrets
      # Downloads the sops binary and the helm-secrets plugin into the shared
      # volume. Pin both versions and record them.
      ...

configs:
  cm:
    helm.valuesFileSchemes: >-
      secrets, secrets+gpg-import, secrets+age-import, https
```

**This is the step most likely to need iteration.** If it fights back, consult
the current `helm-secrets` and Argo CD documentation rather than guessing — the
plugin wiring has changed across Argo CD versions.

**Done 2026-09-20 — the working file is [`argocd-values.yaml`](./argocd-values.yaml),
committed.** It follows the upstream helm-secrets *Option 2: Init Container*
recipe, trimmed to the sops/age backend. Two things the sketch above gets wrong
and the real file gets right, both of which fail quietly:

- `HELM_PLUGINS` is a directory **of** plugins, so it points at
  `/gitops-tools/helm-plugins/`, not at the plugin itself. Pointed one level too
  deep, helm finds no plugins and the `secrets://` scheme fails as an unknown
  protocol rather than as a missing plugin.
- `HELM_SECRETS_VALUES_ALLOW_ABSOLUTE_PATH=true` is required. Argo CD hands value
  files to helm as absolute paths, and helm-secrets rejects those by default.

**Verify:**

```sh
kubectl -n argocd get pods
kubectl -n argocd logs deploy/argocd-repo-server | grep -i sops
```

All pods `Running`, and the repo-server shows no SOPS errors.

Clean logs only prove nothing crashed. **Prove decryption instead** — encrypt a
throwaway file with the committed public recipient and decrypt it with the key
the cluster actually holds:

```sh
printf 'canary: it-decrypts\n' > deploy/probe.sops.yaml   # under deploy/, so
sops --encrypt --in-place deploy/probe.sops.yaml           # .sops.yaml matches
POD=$(kubectl -n argocd get pod -l app.kubernetes.io/name=argocd-repo-server \
  -o jsonpath='{.items[0].metadata.name}')
kubectl -n argocd cp deploy/probe.sops.yaml "argocd/$POD:/tmp/probe.sops.yaml" -c repo-server
kubectl -n argocd exec "$POD" -c repo-server -- sh -c \
  'SOPS_AGE_KEY_FILE=/helm-secrets-private-keys/keys.txt /gitops-tools/sops -d /tmp/probe.sops.yaml'
rm -f deploy/probe.sops.yaml
```

`canary: it-decrypts` is the pass condition. Anything else means the key in the
cluster and the recipient in `.sops.yaml` are not a pair, which no later step
will tell you until a sync fails.

## 5 · Argo Rollouts

```sh
helm install argo-rollouts argo/argo-rollouts -n argocd --version <recorded>
```

**Verify:** `kubectl get crd rollouts.argoproj.io` exists. Without this CRD the
chart's blue-green `Rollout` resources will not apply.

## 6 · StorageClass on `/mnt/drive2`

k3s's `local-path` provisioner writes to `/var/lib/rancher/k3s/storage` by
default. Add a second path so volumes land on the data disk, and expose it as its
own class, so the chart requests a disk **by name** rather than hardcoding a host
path.

```sh
kubectl -n kube-system edit configmap local-path-config
```

Add a `storageClassConfigs` entry mapping `mediadive-data` to
`/mnt/drive2/mediadive`, then restart the provisioner:

```sh
kubectl -n kube-system rollout restart deploy/local-path-provisioner
```

> **This edit is not durable.** `local-path-config` is owned by a k3s *Addon*
> (`objectset.rio.cattle.io/owner-name: local-storage`), so k3s reapplies it
> from `/var/lib/rancher/k3s/server/manifests/local-storage.yaml` on restart and
> on upgrade, dropping the `storageClassConfigs` entry. Nothing breaks loudly:
> volumes already provisioned keep their recorded host path, but the **next**
> PVC on `mediadive-data` silently lands on `/var/lib/rancher/k3s/storage` —
> the OS disk — instead of the data disk. That is exactly the failure the probe
> below is meant to catch, reappearing months later.
>
> Make it durable by editing the addon manifest itself, which needs root:
>
> ```sh
> sudo $EDITOR /var/lib/rancher/k3s/server/manifests/local-storage.yaml
> ```
>
> **Outstanding as of 2026-09-20** — the ConfigMap was patched in place, the
> addon manifest was not. Re-check the ConfigMap after any k3s restart until it
> is done.

Create the class:

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: mediadive-data
provisioner: rancher.io/local-path
reclaimPolicy: Retain          # data outlives an accidental PVC deletion
volumeBindingMode: WaitForFirstConsumer
```

**Verify** with a throwaway PVC — this is the step where a wrong path silently
becomes a wrong disk:

```sh
kubectl -n mediadive apply -f - <<'EOF'
apiVersion: v1
kind: PersistentVolumeClaim
metadata: { name: probe }
spec:
  accessModes: [ReadWriteOnce]
  storageClassName: mediadive-data
  resources: { requests: { storage: 1Gi } }
EOF
kubectl -n mediadive run probe --image=busybox --restart=Never \
  --overrides='{"spec":{"containers":[{"name":"probe","image":"busybox","command":["sh","-c","touch /data/ok && sleep 5"],"volumeMounts":[{"name":"d","mountPath":"/data"}]}],"volumes":[{"name":"d","persistentVolumeClaim":{"claimName":"probe"}}]}}'
ls /mnt/drive2/mediadive        # the volume directory must appear here
kubectl -n mediadive delete pod probe; kubectl -n mediadive delete pvc probe
```

## 7 · Operators

These own CRDs the chart depends on, so they are installed here rather than by
Argo — CRD ordering inside a single sync is fragile.

```sh
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update

helm install cnpg cnpg/cloudnative-pg \
  -n cnpg-system --create-namespace --version <recorded>
```

**Verify:** `kubectl get crd clusters.postgresql.cnpg.io`

## 8 · Observability

```sh
helm install kube-prometheus-stack prometheus-community/kube-prometheus-stack \
  -n observability --version <recorded> \
  --set prometheus.prometheusSpec.retention=7d \
  --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.storageClassName=mediadive-data \
  --set grafana.persistence.enabled=true \
  --set grafana.persistence.storageClassName=mediadive-data

helm install loki grafana/loki -n observability --version <recorded> \
  --set deploymentMode=SingleBinary \
  --set loki.storage.type=filesystem

helm install alloy grafana/alloy -n observability --version <recorded>
```

Retention is pinned at **7 days** deliberately: the default grows without bound
and would eventually fill the data disk regardless of how small the application
is. Alloy is used rather than Promtail — lighter, and Promtail is deprecated.

**Verify:**

```sh
kubectl -n observability get pods
kubectl get crd servicemonitors.monitoring.coreos.com
```

The `ServiceMonitor` CRD is what lets the chart declare scraping for `api` and
`worker`.

## 9 · Tunnel route

The tunnel already exists and the zone is Cloudflare-managed, so **no
`cloudflared tunnel login` is needed** and no DNS record has to be created by
hand — adding a public hostname to a managed tunnel creates the CNAME.

First determine how the existing tunnel is run and record it in the facts table:

```sh
systemctl status cloudflared 2>/dev/null || docker ps | grep cloudflared
```

Then add a public hostname:

| Field | Value |
|---|---|
| Hostname | `mediadive.nandan-hl.dev` |
| Service | `http://traefik.kube-system.svc.cluster.local:80` |

> **Corrected 2026-09-20 — `http://localhost:80` is wrong on this cluster.**
> `cloudflared` runs *in* the cluster, so `localhost` inside that pod is the
> `cloudflared` pod, not the node. A route pointing there registers cleanly and
> then 502s every request. The origin has to be the cluster-internal Traefik
> Service, which is what every other hostname on this tunnel already uses.

- **Dashboard-managed tunnel** → **this is a human step.** Stop and ask; it
  cannot be done from the command line.
- **Config-file-managed tunnel** → add an ingress rule to the tunnel config and
  restart `cloudflared`.

**Resolved 2026-09-20 with no action required.** The tunnel is dashboard-managed,
but a `*.nandan-hl.dev` wildcard route already points at the in-cluster Traefik
Service, and it covers `mediadive.nandan-hl.dev`. Both verifies below pass
already. Confirm it is genuinely Traefik answering and not a Cloudflare error
page — compare the bodies, since both are `404`:

```sh
curl -s https://mediadive.nandan-hl.dev                       # → 404 page not found
curl -s -H 'Host: mediadive.nandan-hl.dev' http://localhost:80 # → identical body
```

If the wildcard is ever narrowed, this becomes a human step again.

**Verify** — a 404 from Traefik is the correct result here, because no
application exists yet. It proves routing works, not that the app does:

```sh
curl -s -o /dev/null -w '%{http_code}\n' -H 'Host: mediadive.nandan-hl.dev' http://localhost:80
curl -s -o /dev/null -w '%{http_code}\n' https://mediadive.nandan-hl.dev
```

## 10 · Turn on GitOps

```sh
kubectl apply -f deploy/argocd/application.yaml
```

**Verify:**

```sh
kubectl -n argocd get application mediadive
```

`Synced` with zero resources is the expected result while the chart is still
empty — and it is exactly what this step proves: Argo can read the repository,
resolve the chart, and reconcile. Everything from here arrives by commit.

## 11 · Restore test — mandatory, after the chart defines Postgres

Not part of bootstrap proper: run this once the chart brings up the Postgres
cluster and its first backup has completed.

Because the database and its backups share one disk, **the value of the backup is
entirely in whether the restore procedure is known to work.** An untested backup
is not a backup.

```sh
kubectl -n mediadive get cluster                      # Postgres cluster healthy
kubectl -n mediadive get backup                       # at least one completed
# Restore into a NEW cluster from the latest backup, confirm the schema is
# present, then delete the restored cluster.
```

Record the outcome in the completion record. If the restore does not work, that
is a blocking defect, not a to-do.

---

## Re-running steps

| Step | Safe to re-run |
|---|---|
| 1 Confirm · 2 Namespaces | yes |
| 3 age key | **no** — regenerating orphans every encrypted value |
| 4–8 helm installs | yes, as `helm upgrade --install` |
| 6 StorageClass probe | yes |
| 9 Tunnel route | yes, idempotent |
| 10 Application | yes |

## When a step does not verify

Stop at the failing step; do not proceed. Later steps assume earlier ones. Record
what failed and the actual output — a half-bootstrapped cluster that appears to
work is worse than one that clearly does not.

## Handoff — commit back

- `.sops.yaml` with the age **public** recipient
- This file, with the facts table filled in and versions recorded
- Any SOPS-encrypted secrets created for the chart
- The completion record below

**Never commit:** the age private key. **Never push to `main`** — open a pull
request. **Do not modify application code**; this session's scope is
infrastructure.

## Completion record

```
Date:                         2026-09-20
Node name:                    homelab            k3s version: v1.35.5+k3s1
Argo CD:                      10.9.2 (v3.5.3)    Argo Rollouts: 2.43.2 (v1.10.0)
CloudNativePG:                0.29.0 (1.30.0)    Prometheus stack: 91.4.1 (v0.94.0)
Loki:                         7.3.0 (3.6.12)     Alloy: 1.12.1 (v1.19.2)
cloudflared:                  container (in-cluster Deployment) — dashboard-managed
StorageClass probe:           passed
Tunnel verify (local / external):  404 / 404, identical bodies — Traefik answering
Argo Application status:      Synced / Healthy, zero resources
Restore test (step 11):       not yet run — the chart defines no Postgres yet
```

### Deviations from this runbook

1. **Steps 4 and 8 use committed values files, not `--set` flags.** `helm-secrets`
   needs more wiring than a flag list, and the Loki chart refuses to install
   without an explicit `schemaConfig`. All four are in `deploy/`:
   `argocd-values.yaml`, `kube-prometheus-stack-values.yaml`, `loki-values.yaml`,
   `alloy-values.yaml`. Every downloaded binary version in them is pinned.

2. **`vals` and `kubectl` are not installed on the repo-server.** Upstream's
   example downloads both; they serve the `vals` backend and the
   `secrets+*-import-kubernetes://` schemes, neither of which this repository
   uses. Add them back if either is ever adopted.

3. **kube-prometheus-stack has the k3s control-plane scrape targets disabled**
   (`kubeControllerManager`, `kubeScheduler`, `kubeProxy`, `kubeEtcd`). k3s runs
   the control plane in one process rather than as static pods, so these targets
   do not exist; left enabled they alert permanently, which teaches everyone to
   ignore the alert list. Prometheus also gained `retentionSize: 20GB` alongside
   the 7-day window — time-based retention alone does not bound a disk.

4. **Alloy was given a config.** The chart ships an empty one, so a default
   install runs, reports healthy, and collects nothing. It reads pod logs through
   the Kubernetes API rather than tailing `/var/log/pods`, so it needs no
   hostPath mount.

5. **`helm`, `sops` and `age` were installed into `~/.local/bin`, not system-wide**
   — `sudo` needs a password in this session. They add no service and no
   listening port, so the host inventory in `~/CLAUDE.md` is unchanged. `age` is
   also available from `apt`.

6. **Step 6's ConfigMap edit is not yet durable.** See the warning in that step:
   it needs a root edit of the k3s addon manifest, which has not been done.

### Outstanding after this run

| Item | Why it matters |
|---|---|
| Persist `storageClassConfigs` in `/var/lib/rancher/k3s/server/manifests/local-storage.yaml` | Needs root. Until then a k3s restart silently sends the next `mediadive-data` PVC to the OS disk. |
| Restore test (step 11) | Mandatory, blocked until the chart defines Postgres. The backup shares a disk with the database, so the procedure working *is* the backup. |
| Argo CD has no ingress | Reachable only via `kubectl port-forward svc/argocd-server -n argocd 8080:443`. `server.insecure` is already set, so an Ingress on the wildcard is a small addition if it is wanted. |
| Grafana has no ingress | Same; `kubectl port-forward svc/kube-prometheus-stack-grafana -n observability 3000:80`. |

### A note on `/mnt/drive2` and Samba

`~/CLAUDE.md` records that the host exports `/mnt/drive2` read-write over SMB as
`[media2]`. That is worth knowing here, but it does **not** reach mediadive's
data: the local-path provisioner creates `/mnt/drive2/mediadive` as `root:root`
mode `0700`, so the `kujoforall` account the share authenticates as cannot
descend into it. `/mnt/drive2/personal` is a different matter and is not this
project's concern. If that directory's mode ever changes, this stops being true.
