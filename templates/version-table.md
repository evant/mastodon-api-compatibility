{% macro version(version) -%}
{{ version.version }}
{%- if version.note -%}
<span data-tooltip="{{ version.note }}"><i class="icon iinfo"></i></span>
{%- endif -%}
{%- if version.deprecated -%}
<span data-tooltip="deprecated in {{ version.deprecated }}"><i class="icon iwarning"></i></span>
{%- endif -%}
{%- if version.removed -%}
<span data-tooltip="removed in {{ version.removed }}"><i class="icon ierror"></i></span>
{%- endif -%}
{%- endmacro version %}

{% macro table(name, software, entities, values, link="") -%}
<table>
<thead><tr><td>
{{- name }}{% if link %} {{ link }}{% endif -%}
</td>
{%- for software in software %}<td>{{ software.name }}</td>{% endfor -%}
</tr></thead>
<tbody>
{%- for name, value in values -%}
<tr>
<td>
{{- name }} {% if value.entity -%}
{%- set entity = entities[value.entity] -%}
<a href="{{ entity.path }}.html">{{ entity.name }}</a>
{%- endif %}</td>
{%- for software in software -%}
<td>{%- if software.key in value.software -%}
{{ self::version(version=value.software[software.key]) }}
{%- endif -%}
</td>
{%- endfor -%}
</tr>
{%- endfor -%}
</tbody>
</table>
{%- endmacro table %}
