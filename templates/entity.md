{% import "software-table.md" as software %}
{% import "version-table.md" as version %}

# {{ entity.name }}

{{ software::table(software=software, values=entity.software) }}

{% if entity.attributes | length > 0 %}
{{ version::table(name="Attributes", software=software, entities=entities, values=entity.attributes) }}
{% endif %}

{% if entity.values | length > 0 %}
{{ version::table(name="Values", software=software, entities=entities, values=entity.values) }}
{% endif %}
