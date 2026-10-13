# TODO: EPUB CSS Support

**Status: ⚠️ Partial / Not production-ready.** The CSS engine added to the EPUB
renderer works for simple cases but does **not** correctly style many real-world
ebooks. It should not be considered finished.

All of the code discussed here lives in `source/epub/muepub.c`.

---

## 1. Why we added CSS support

The EPUB renderer originally ignored stylesheets and hard-coded a small set of
styles (headings, `<b>`, `<i>`, `<pre>`, lists, blockquotes). That was fine for
plain books, but many professionally-produced EPUBs rely on their stylesheet to
position text — most visibly **text alignment**.

The concrete trigger was this markup from a real book (an O'Reilly / Safari
"`sbo-rt-content`" style EPUB):

```html
<p data-type="attribution">
  Mayo Oshin, founder and CEO, Siennai Analytics,
  <span class="keep-together">early LangChain contributor</span>
</p>
```

styled by:

```css
#sbo-rt-content blockquote p[data-type="attribution"] {
  font-style: normal;
  margin-top: 5px;
}
```

The attribution line is supposed to be **right-aligned** (a pull-quote credit).
In the current build it is **NOT right-aligned** — it renders as ordinary
left-aligned body text. See section 4 for exactly why.

---

## 2. What was implemented (approach)

We wrote a **small, hand-rolled CSS engine** — deliberately not a full CSS
parser — that runs while walking each chapter's XHTML.

### 2.1 Data model

* `epub_css_rule` — one rule = a single **compound selector** plus the handful of
  declarations we understand:
  * `text-align` → `left | center | right | justify`
  * `font-style` → `normal | italic`
  * `font-weight` → `normal | bold`
  * `font-size` (pt/px/em/rem/ex/in/cm/mm/pc, converted to points)
  * `margin-top` / `margin-bottom` (same units)
* Selector parts supported: element (`p`), class (`.cls`), id (`#id`), and a
  single attribute selector (`[attr]` or `[attr=value]`).
* Each rule carries a **specificity** score (id = 100, class/attr = 10, element
  = 1) and a **source-order** index, so the most specific + last-defined rule
  wins.

### 2.2 Rule loading

Stylesheets are collected before the body is walked:

* inline `<style>…</style>` in `<head>` (`epub_css_parse`)
* `<link rel="stylesheet" href="…">` resolved relative to the chapter directory
  (`epub_load_stylesheet`)
* the attribute string of an inline `style="…"` on any element
  (`epub_apply_inline_style`, applied last = highest priority)

### 2.3 Matching + application

* The walker keeps an **ancestor stack** (`epub_elem stack[]`, push in
  `epub_push`, pop in `epub_pop`).
* For each element, `epub_css_match` finds the best-matching rule and
  `epub_apply_css` copies its alignment/font/margin onto the current block.
* Alignment is threaded through the existing `p->center` / `p->right` fields
  onto `epub_block.center` / `epub_block.right`, which the layout pass already
  knows how to render (see `epub_layout_block` → the `b->center` / `b->right`
  branches).

<!-- CONTINUE-TODO -->

---

## 3. What works (verified)

* Compound selectors on a **single** element:
  * `p[data-type="attribution"]` → right-aligned ✔
  * `.centered` → centered ✔
  * `#byid` → centered ✔
  * `p` element selector ✔
* Inline `style="text-align:right"` ✔ (highest priority)
* `text-align`, `font-style`, `font-weight`, `font-size`, `margin-top/bottom`
* Loading from `<style>`, `<link>`, and inline `style=`

Two real bugs were found and fixed while building this (kept here for context):

* **Infinite loop** in the selector parser: the tag branch didn't skip
  whitespace, so a selector with a trailing space before `{` (e.g. `#byid `)
  spun forever and hung the render.
* **Use-after-free / segfault** in `epub_load_stylesheet`: it freed a pointer it
  then read in the error handler, and freed uninitialized `data` when the linked
  file was missing. A missing stylesheet now just warns and continues.

---

## 4. What is still broken (the important part)

### 4.1 THE headline bug: the attribution is NOT right-aligned in the real book

The example in section 1 is **still rendered as left-aligned body text**. The
reason is the selector:

```css
#sbo-rt-content blockquote p[data-type="attribution"] { … }
```

This is a **descendant (complex) selector** — four simple selectors joined by
descendant combinators. Our parser **deliberately skips** any selector that
contains a combinator. In `epub_css_parse`:

```c
for (i = 0; i < sel_len; i++) {
    char c = sel_start[i];
    if (c == '>' || c == '+' || c == '~' || c == ':' || c == ',')
        skip = 1;
}
```

When `skip` is set, the rule's body is consumed and the rule is thrown away
(`doc->css_rule_count--`). So `#sbo-rt-content blockquote p[data-type="attribution"]`
is dropped entirely, and the `<p data-type="attribution">` gets **no** style at
all — hence no right alignment.

> Note: our own tests "passed" because they used a simplified selector,
> `p[data-type="attribution"] { text-align:right; }`, which is a single compound
> selector and therefore *is* handled. The real book's descendant selector is
> not. This is a testing gap, not just a feature gap.

Also note the `font-style: normal` in that same rule does not apply either —
though that particular declaration has no visible effect here because nothing
made the attribution italic in the first place.

### 4.2 Other known gaps

* **Descendant / child / sibling combinators** (`a b`, `a > b`, `a + b`,
  `a ~ b`) are unsupported and silently dropped. This is the single biggest
  reason real books don't style correctly — most EPUB stylesheets are written
  almost entirely with descendant selectors.
* **Selector lists** (`a, b, c`) are dropped (the `,` triggers `skip`).
* **Pseudo-classes / pseudo-elements** (`:first-child`, `::before`,
  `:not(...)`) are dropped.
* **No inheritance / cascade across elements.** A style set on an ancestor
  (e.g. `#sbo-rt-content { font-family: … }`) is not inherited by descendants.
  We only apply a rule whose selector matches the *current* element.
* **No box model.** `margin`, `padding`, `width`, `float`, `display`,
  `text-indent`, `line-height` are only partially handled (margins) or ignored.
* **`text-align: justify`** is treated as plain left (the `align == 4` case just
  clears center/right); there is no real justification / word-spacing.
* **`@media`, `@font-face`, `@import`** are skipped by the `@`-rule handler.
* **Specificity is per-simple-selector only.** There is no notion of comparing a
  descendant selector's specificity, because descendant selectors don't match at
  all.
* Colors, backgrounds, borders, and font-family are not applied at all.

<!-- CONTINUE-TODO -->
