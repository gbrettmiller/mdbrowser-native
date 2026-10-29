# Front matter edge cases

Small documents that pin down how front matter is read. Each one renders; none should show its front matter as text.

- [Title over heading](title-over-heading.md): front matter `title` wins over the first heading.
- [Malformed YAML](malformed.md): the block is hidden, and the title falls back to the first heading.
- [Unknown keys](unknown-keys.md): keys from other tools are ignored.
- [Empty block](empty.md): `---` straight after `---` is still front matter, not two rules.
- [Not at the top](not-at-top.md): a `---` block later in a document is a rule and a heading, never front matter.
