#!/usr/bin/env python3
"""
anki_builder -- reusable helpers for building Anki .apkg decks from medical
exam-revision notes.

Exposes:
  - DeckBuilder : registers topic subdecks, adds Basic (Q/A) and Cloze notes
                  with consistent styling and tags, then writes a .apkg.
  - safe()      : HTML-sanitises card text so stray '<', '>' and '&'
                  (e.g. "CD4 <200", "M > F", "<CF", "rods & cones") render as
                  literal text. Without this they are swallowed by Anki's
                  Chromium-based renderer as malformed tags and the rest of the
                  card silently disappears -- the single most common way a
                  medical deck looks fine in the build log but is broken in Anki.

Requires: genanki   (pip install genanki --break-system-packages)

Stable IDs
----------
Model and deck IDs are derived deterministically from the parent deck name
(and the subdeck key). Re-running the builder on the same deck and re-importing
into Anki therefore UPDATES the existing notes instead of creating duplicates,
which is what a student re-importing an edited deck expects.
"""

import hashlib
import genanki


def _stable_id(*parts):
    """Deterministic positive int id from strings (genanki wants an int id)."""
    h = hashlib.sha256("::".join(parts).encode("utf-8")).hexdigest()
    return int(h[:12], 16) % (1 << 31) + 1


# ---------------------------------------------------------------------------
# HTML escaping
# ---------------------------------------------------------------------------
# Inline tags we intentionally allow in card content. Everything else carrying
# an angle bracket is escaped to literal text. Keep this list small and only
# for genuine formatting -- if you need a tag that is not here, add it here
# rather than disabling safe().
_KNOWN_TAGS = [
    "<b>", "</b>", "<i>", "</i>", "<u>", "</u>",
    "<ul>", "</ul>", "<ol>", "</ol>", "<li>", "</li>",
    "<br>", "<br/>", "<sub>", "</sub>", "<sup>", "</sup>",
    "<table>", "</table>", "<tr>", "</tr>", "<td>", "</td>",
    "<th>", "</th>", "<thead>", "</thead>", "<tbody>", "</tbody>",
]


def safe(s):
    if s is None:
        return ""
    s = str(s)
    placeholders = {}
    for i, tag in enumerate(_KNOWN_TAGS):
        ph = f"\x00{i}\x00"
        placeholders[ph] = tag
        s = s.replace(tag, ph)
    s = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    for ph, tag in placeholders.items():
        s = s.replace(ph, tag)
    return s


# ---------------------------------------------------------------------------
# Shared card styling. Left-aligned for prose-heavy medical cards; the key
# answer term is highlighted via <b> so it stands out at a glance during
# rapid review.
# ---------------------------------------------------------------------------
_BASIC_CSS = """
.card { font-family: -apple-system, Segoe UI, Roboto, Arial, sans-serif;
        font-size: 19px; color: #1a1a2e; background: #fbfbfd; text-align: left;
        line-height: 1.5; padding: 14px 18px; }
.q { font-weight: 600; }
.a { color: #16213e; }
hr#answer { border: none; border-top: 2px solid #5a7d9a; margin: 12px 0; }
b, strong { color: #c2185b; }
ul, ol { margin: 6px 0 6px 0; padding-left: 22px; }
li { margin: 2px 0; }
table { border-collapse: collapse; margin: 8px 0; }
td, th { border: 1px solid #c5ccd6; padding: 4px 8px; text-align: left; }
th { background: #eef2f7; }
.muted { color: #6b7280; font-size: 15px; }
"""

_CLOZE_CSS = """
.card { font-family: -apple-system, Segoe UI, Roboto, Arial, sans-serif;
        font-size: 19px; color: #1a1a2e; background: #fbfbfd; text-align: left;
        line-height: 1.5; padding: 14px 18px; }
.cloze { font-weight: bold; color: #c2185b; }
.muted { color: #6b7280; font-size: 15px; }
ul, ol { margin: 6px 0 6px 0; padding-left: 22px; }
li { margin: 2px 0; }
hr { border: none; border-top: 1px solid #d0d0d8; margin: 10px 0; }
"""


class DeckBuilder:
    """Accumulates notes across topic subdecks and writes a single .apkg.

    Typical use:
        b = DeckBuilder("Cardiology Revision", base_tag="Cardiology")
        b.add_deck("arrhythmia", "01 Arrhythmias")
        b.basic("arrhythmia", "First-line drug in stable VT?", "<b>Amiodarone</b>.", ["VT"])
        b.cloze("arrhythmia", "Beck's triad: {{c1::hypotension}}, {{c2::muffled heart sounds}}, {{c3::raised JVP}}.", ["Tamponade"])
        stats = b.build("/mnt/user-data/outputs/Cardiology.apkg")
    """

    def __init__(self, parent_name, base_tag="Revision"):
        self.parent_name = parent_name
        self.base_tag = base_tag
        self.note_count = 0
        self.decks = {}
        self._order = []
        self.basic_model = genanki.Model(
            _stable_id(parent_name, "basic-model"),
            "Med Basic",
            fields=[{"name": "Front"}, {"name": "Back"}],
            templates=[{
                "name": "Card 1",
                "qfmt": "<div class='q'>{{Front}}</div>",
                "afmt": "{{FrontSide}}<hr id='answer'><div class='a'>{{Back}}</div>",
            }],
            css=_BASIC_CSS,
        )
        self.cloze_model = genanki.Model(
            _stable_id(parent_name, "cloze-model"),
            "Med Cloze",
            fields=[{"name": "Text"}, {"name": "Extra"}],
            model_type=genanki.Model.CLOZE,
            templates=[{
                "name": "Cloze",
                "qfmt": "<div class='q'>{{cloze:Text}}</div>",
                "afmt": "<div class='q'>{{cloze:Text}}</div>"
                        "{{#Extra}}<hr><div class='muted'>{{Extra}}</div>{{/Extra}}",
            }],
            css=_CLOZE_CSS,
        )

    def add_deck(self, key, name):
        """Register a subdeck. `name` becomes 'Parent::name' in Anki."""
        deck_id = _stable_id(self.parent_name, "deck", key)
        self.decks[key] = genanki.Deck(deck_id, f"{self.parent_name}::{name}")
        self._order.append(key)
        return key

    def _resolve(self, key):
        if key not in self.decks:
            # Auto-create so a typo or an un-registered key never crashes a
            # long build half-way through; the subdeck is just named after key.
            self.add_deck(key, key)
        return self.decks[key]

    def basic(self, deck_key, front, back, tags=None):
        note = genanki.Note(
            model=self.basic_model,
            fields=[safe(front), safe(back)],
            tags=[self.base_tag] + list(tags or []),
        )
        self._resolve(deck_key).add_note(note)
        self.note_count += 1

    def cloze(self, deck_key, text, tags=None, extra=""):
        """`text` MUST contain at least one {{cN::...}} deletion."""
        note = genanki.Note(
            model=self.cloze_model,
            fields=[safe(text), safe(extra)],
            tags=[self.base_tag] + list(tags or []),
        )
        self._resolve(deck_key).add_note(note)
        self.note_count += 1

    def build(self, out_path):
        decks = [self.decks[k] for k in self._order if len(self.decks[k].notes) > 0]
        genanki.Package(decks).write_to_file(out_path)
        return {
            "path": out_path,
            "notes": self.note_count,
            "decks": [(d.name, len(d.notes)) for d in decks],
        }