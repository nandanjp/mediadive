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
| Cluster state at start | Empty; nothing else deployed |
| Node name | _to record_ |
| k3s version | _to record_ |
| cloudflared location | _to record: host service, or container_ |
| cloudflared management | _to record: local config file, or Cloudflare dashboard_ |

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
| Argo CD | `argo/argo-cd` | `helm search repo argo/argo-cd --versions \| head -3` | _record_ |
| Argo Rollouts | `argo/argo-rollouts` | `helm search repo argo/argo-rollouts --versions \| head -3` | _record_ |
| CloudNativePG | `cnpg/cloudnative-pg` | `helm search repo cnpg/cloudnative-pg --versions \| head -3` | _record_ |
| Prometheus stack | `prometheus-community/kube-prometheus-stack` | `helm search repo prometheus-community/kube-prometheus-stack --versions \| head -3` | _record_ |
| Loki | `grafana/loki` | `helm search repo grafana/loki --versions \| head -3` | _record_ |
| Alloy | `grafana/alloy` | `helm search repo grafana/alloy --versions \| head -3` | _record_ |

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

**Verify:**

```sh
kubectl -n argocd get pods
kubectl -n argocd logs deploy/argocd-repo-server | grep -i sops
```

All pods `Running`, and the repo-server shows no SOPS errors. A real decryption
test comes in step 9, when the chart first contains an encrypted value.

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
| Service | `http://localhost:80` (Traefik binds the node's port 80) |

- **Dashboard-managed tunnel** → **this is a human step.** Stop and ask; it
  cannot be done from the command line.
- **Config-file-managed tunnel** → add an ingress rule to the tunnel config and
  restart `cloudflared`.

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
Date:
Node name:                    k3s version:
Argo CD:                      Argo Rollouts:
CloudNativePG:                Prometheus stack:
Loki:                         Alloy:
cloudflared:                  host service / container — config file / dashboard
StorageClass probe:           passed / failed
Tunnel verify (local / external):
Argo Application status:
Restore test (step 11):       passed / failed / not yet run
Deviations from this runbook:
```
