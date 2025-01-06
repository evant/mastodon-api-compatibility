{% import "software-table.md" as software %}
{% import "version-table.md" as version %}

# {{ title | title }}

{% for api in apis %}
## {{ api.request }}
{{ software::table(software=software, values=api.software) }}

{%- if api.params | length > 0 -%}
{{ version::table(name="Parameters", software=software, entities=entities, values=api.params) }}
{%- endif -%}

{%- if api.responses | length > 0 -%}
{%- for response in api.responses -%}
{{- self::response(software=software, entities=entities, response=response) -}}
{%- endfor -%}
{%- endif %}

{% endfor -%}

{% macro response(software, entities, response) -%}
{%- set_global name = "Response" -%}
{%- set_global link = "" -%}
{%- if response.value is string -%}
{%- set entity = entities[response.value] -%}
{%- set_global values = entity.attributes -%}
{%- if response.array -%}
{%- set_global link = "<a href='" ~ entity.path ~ ".html'>" ~ entity.name ~ "[]</a>" -%}
{%- else %}
{%- set_global link = "<a href='" ~ entity.path ~ ".html'>" ~ entity.name ~ "</a>" -%}
{%- endif %}
{%- else %}
{%- set http_code = response.http_code | default(value="") -%}
{%- set_global name = http_code ~ " Response" | trim -%}
{%- set_global values = response.value -%}
{%- endif -%}
{{- version::table(name=name, link=link, software=software, entities=entities, values=values) -}}
{%- endmacro response %}
