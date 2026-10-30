# Front matter edge cases

Small documents that pin down how front matter is read. Each one renders; none should show its front matter as text.

- [Title over heading](title-over-heading.md): front matter `title` wins over the first heading.
- [Malformed YAML](malformed.md): the block is hidden, and the title falls back to the first heading.
- [Unknown keys](unknown-keys.md): keys from other tools are ignored.
- [Empty block](empty.md): `---` straight after `---` is still front matter, not two rules.
- [Not at the top](not-at-top.md): a `---` block later in a document is a rule and a heading, never front matter.
- [A byline](byline.md): author, date and site name under the heading, then the description.
- [A repeated description](description-repeated.md): shown once when the post opens with it.
- [A date we can't read](date-as-written.md): shown as written.
- [Wrong kinds of value](wrong-kinds.md): dropped one by one; unknown keys and the image fields add nothing.
- [An enclosure](enclosure.md): a Listen line that opens in the system.
- [An enclosure we refuse](bad-enclosure.md): not a web address, so no line.
- [A three-post blog](blog/one.md): Previous and Next, and a Next that isn't there.
