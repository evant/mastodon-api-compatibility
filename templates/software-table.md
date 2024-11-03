{% import "version-table.md" as table %}

{% macro table(software, values) -%}
<table>
<thead><tr><td>Software</td><td>Version</td></tr></thead>
<tbody>
{%- for software in software -%}
<tr>
{%- if software.key in values -%}
<td>
{%- set version = values[software.key] -%}
{%- if version is object -%}
<a href="{{ version.docs | default(value=software.docs) }}">{{ software.name }}</a>
{%- else -%}
<a href="{{ software.docs }}">{{ software.name }}</a>
{%- endif -%}
</td>
<td>{{ table::version(version=version) }}</td>
{%- else -%}
<td>{{ software.name }}</td><td></td>
{%- endif -%}
</tr>
{%- endfor -%}
</tbody>
</table>
{%- endmacro table %}
