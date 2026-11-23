"""Refait data/trinkets.yaml : stats des Trinkets (bgbuddy.gg) + infos des cartes (HearthstoneJSON).

Usage :  python tools/make_trinkets.py [tools/bgbuddy_trinkets.txt] [chemin de cards.json]

tools/bgbuddy_trinkets.txt = une ligne par Trinket de https://bgbuddy.gg/trinkets (Solo, tous rangs, patch actuel),
au format  nom|L ou G|placement moyen|% de choix|parties  (L = Lesser, G = Greater). À recopier à chaque patch.
Les Trinkets absents de la page ne sont pas dans le pool actuel : ils ne sont pas écrits.
"""
import json
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
SRC = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "tools" / "bgbuddy_trinkets.txt"
CARDS = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "data" / "cards.json"
OUT = ROOT / "data" / "trinkets.yaml"

# Costs changed by a server-side hotfix that HearthstoneJSON's cards.json does not show yet (it follows
# client patches only). Remove a line once cards.json has the same cost.
# 37.0.3 hotfix, 2026-11-17: https://us.forums.blizzard.com/en/hearthstone/t/3703-hotfix-patch/168317
COST_FIXES = {  # (name, L or G): cost
    ("Training Certificate", "L"): 5, ("Bloodbound Earrings", "L"): 3, ("Vibrant Bubble", "L"): 4, ("Ruby Tusk", "L"): 1,
    ("Jailer Sticker", "L"): 3, ("Demonic Bloodletter", "L"): 1, ("Hoggy Bank", "L"): 0, ("Wand of Divination", "L"): 1,
    ("Reusable Batteries", "L"): 0, ("Baller Portrait", "L"): 0, ("Cursed Crystal", "L"): 0, ("Fish Portrait", "L"): 0,
    ("Mindslasher Portrait", "L"): 2,
    ("Blood Golem Sticker", "G"): 7, ("Snarky Portrait", "G"): 5, ("Soul Fermenter", "G"): 7, ("Trailblazer Sticker", "G"): 6,
    ("Weighted Gauntlet", "G"): 5, ("Volumizer Portrait", "G"): 0, ("Sinstone Sticker", "G"): 1, ("Aggem Sticker", "G"): 0,
    ("Sky Golem Portrait", "G"): 0, ("Portable Factory", "G"): 1, ("Nerglish Phrasebook", "G"): 0,
}

HEADER = """\
# Trinkets of the current pool: stats + card info. File made by tools/make_trinkets.py (don't edit by
# hand: edit tools/bgbuddy_trinkets.txt then run the tool again).
#
# Stats source: https://bgbuddy.gg/trinkets (Firestone community data, Solo, all ranks, current patch,
# read on 2026-11-07, season 14, patch 37.0.2: about 4 days of games, so most rows have under 5,000).
# Costs include the 37.0.3 server hotfix (2026-11-17), which cards.json does not show: see COST_FIXES in the tool.
# Trinkets bgbuddy shows no 37.0.2 games for keep their 36.6.x row (read on 2026-11-02).
#   avg   = average place of the players who took it (lower = better)
#   pick  = % of times it is picked when offered
#   games = number of games (under 500 = not very reliable)
# Card fields: kind (lesser / greater), cost (base price in gold; -2 for a "pivot" Trinket),
# races (linked tribe: offered mostly if you have some on board), gives (minion given by a Portrait / Sticker).
"""


def main():
    cards = json.loads(CARDS.read_text(encoding="utf-8"))
    by_dbf = {c.get("dbfId"): c for c in cards}
    trinkets = {}
    for c in cards:
        if c.get("type") == "BATTLEGROUND_TRINKET" and not c.get("isBattlegroundsDuosExclusive"):
            trinkets[(c["name"], c["spellSchool"][0])] = c
    out = []
    for line in SRC.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        name, kind, avg, pick, games = line.split("|")
        c = trinkets.get((name, kind))
        if c is None:
            print("not found:", name, kind)
            continue
        entry = {"id": c["id"], "name": name, "kind": "lesser" if kind == "L" else "greater", "cost": COST_FIXES.get((name, kind), c.get("cost", 0)),
                 "avg": float(avg), "pick": float(pick), "games": int(games)}
        if c.get("battlegroundsAssociatedRaces"):
            entry["races"] = c["battlegroundsAssociatedRaces"]
        rel = by_dbf.get(c.get("battlegroundsRelatedCard"))
        if rel and rel.get("type") == "MINION":
            entry["gives"] = rel["name"]
        out.append(entry)
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(HEADER)
        yaml.safe_dump(out, f, allow_unicode=True, sort_keys=False, width=200)
    print(len(out), "trinkets ->", OUT)


if __name__ == "__main__":
    main()
