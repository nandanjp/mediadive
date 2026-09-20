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
