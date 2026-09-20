{{/* Base labels on every object. */}}
{{- define "mediadive.labels" -}}
app.kubernetes.io/part-of: mediadive
app.kubernetes.io/managed-by: {{ .Release.Service }}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version }}
{{- end }}

{{/*
Selector labels for one component. Call with (dict "name" "api").

Deliberately just the name: pod templates apply these alongside
`mediadive.labels`, and any key present in both would render twice into the same
map — invalid YAML that `helm lint` accepts and the API server rejects.
*/}}
{{- define "mediadive.selector" -}}
app.kubernetes.io/name: {{ .name }}
{{- end }}

{{/* Image reference for one component. Call with (dict "root" . "name" "api"). */}}
{{- define "mediadive.image" -}}
{{ .root.Values.image.registry }}/mediadive-{{ .name }}:{{ .root.Values.image.tag }}
{{- end }}

{{/* Whether application components should render — see image.tag in values. */}}
{{- define "mediadive.appDeployable" -}}
{{- ne .Values.image.tag "unset" -}}
{{- end }}

{{/*
Read a required secret value, failing with an actionable message when the
encrypted values file is missing or incomplete. Without this, a missing
secrets.sops.yaml surfaces as a nil-pointer error naming a template line rather
than the value that is absent.

Call with (dict "root" . "path" "garage.rpcSecret").
*/}}
{{- define "mediadive.secret" -}}
{{- $value := "" -}}
{{- if .root.Values.secrets -}}
{{- $value = dig (splitList "." .path | first) (splitList "." .path | last) "" .root.Values.secrets -}}
{{- end -}}
{{- if not $value -}}
{{- fail (printf "secrets.%s is required but not set. Decrypt secrets.sops.yaml, or see deploy/charts/mediadive/secrets.example.yaml for the expected shape." .path) -}}
{{- end -}}
{{- $value -}}
{{- end }}
