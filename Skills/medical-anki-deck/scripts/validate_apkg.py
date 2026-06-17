#!/usr/bin/env python3
"""
validate_apkg -- prove a .apkg actually imports into Anki before you hand it over.

genanki will happily write a file that is malformed in ways it does not detect;
the only reliable check is to feed the file to Anki's own engine. This does two
levels:

  1. Structural : the file is a valid zip, contains collection.anki2, and that
                  database has the tables Anki expects.
  2. True import: spin up a fresh, empty collection with the official `anki`
                  package and import the deck into it -- the gold-standard test
                  that Anki itself accepts the file. Reports notes/cards created
                  and the deck tree.

Usage:
    python validate_apkg.py path/to/deck.apkg

Requires: anki   (pip install anki --break-system-packages)
Exit code: 0 = PASS, 1 = FAIL.
"""

import os
import sys
import shutil
import zipfile
import sqlite3
import tempfile

REQUIRED_TABLES = {"col", "notes", "cards", "revlog", "graves"}


def structural(apkg):
    print("=== STRUCTURAL ===")
    if not os.path.exists(apkg):
        print(f"FAIL: file not found: {apkg}")
        return False
    print("size:", os.path.getsize(apkg), "bytes")
    if not zipfile.is_zipfile(apkg):
        print("FAIL: not a valid zip archive")
        return False
    with zipfile.ZipFile(apkg) as z:
        names = z.namelist()
        print("members:", names)
        db_member = "collection.anki21" if "collection.anki21" in names else "collection.anki2"
        if db_member not in names:
            print("FAIL: no collection.anki2 / .anki21 inside the package")
            return False
        tmp = tempfile.mkdtemp()
        try:
            z.extract(db_member, tmp)
            con = sqlite3.connect(os.path.join(tmp, db_member))
            tables = {r[0] for r in con.execute(
                "SELECT name FROM sqlite_master WHERE type='table'")}
            con.close()
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
        missing = REQUIRED_TABLES - tables
        if missing:
            print("FAIL: missing required tables:", missing)
            return False
    print("PASS: structural checks OK")
    return True


def true_import(apkg):
    print("\n=== TRUE IMPORT (official anki engine) ===")
    try:
        from anki.collection import Collection
    except ImportError:
        print("SKIP: `anki` package not installed "
              "(pip install anki --break-system-packages) -- "
              "structural check still ran.")
        return None

    tmp = tempfile.mkdtemp()
    col = Collection(os.path.join(tmp, "fresh.anki2"))
    try:
        print("before -> notes:", col.note_count(), "cards:", col.card_count())
        try:
            from anki.collection import (
                ImportAnkiPackageRequest, ImportAnkiPackageOptions)
            col.import_anki_package(ImportAnkiPackageRequest(
                package_path=apkg, options=ImportAnkiPackageOptions()))
            via = "import_anki_package"
        except Exception:
            from anki.importing.apkg import AnkiPackageImporter
            imp = AnkiPackageImporter(col, apkg)
            imp.run()
            via = "AnkiPackageImporter (legacy)"
        notes, cards = col.note_count(), col.card_count()
        print(f"imported via {via}")
        print("after  -> notes:", notes, "cards:", cards)
        decks = sorted(d.name for d in col.decks.all_names_and_ids())
        print("decks:", len(decks))
        for d in decks:
            print("   ", d)
        if notes == 0 or cards == 0:
            print("FAIL: import produced no notes/cards")
            return False
    finally:
        col.close()
        shutil.rmtree(tmp, ignore_errors=True)
    print("PASS: imports cleanly into a fresh Anki collection")
    return True


def main():
    if len(sys.argv) != 2:
        print("usage: python validate_apkg.py path/to/deck.apkg")
        sys.exit(2)
    apkg = sys.argv[1]
    ok_struct = structural(apkg)
    ok_import = true_import(apkg)
    # SKIP (None) from the import stage is tolerated; an explicit False is not.
    passed = ok_struct and (ok_import is not False)
    print("\n" + ("PASS" if passed else "FAIL"))
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()