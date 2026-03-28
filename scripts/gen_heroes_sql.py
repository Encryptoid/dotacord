import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent
HEROES_JSON = PROJECT_DIR / "data" / "heroes.json"
OUTPUT_SQL = PROJECT_DIR / "sql" / "hydrate_heroes.sql"


def main():
    with open(HEROES_JSON) as f:
        heroes = json.load(f)

    lines = []
    missing = []

    for hero in heroes:
        hero_id = hero["id"]
        name = hero["localized_name"].replace("'", "''")
        roles = hero.get("roles", [])

        is_carry = 1 if "Carry" in roles else 0
        is_support = 1 if "Support" in roles else 0

        if is_carry == 0 and is_support == 0:
            missing.append(hero["localized_name"])
            continue

        lines.append(
            f"INSERT OR IGNORE INTO heroes (hero_id, name, is_carry, is_mid, is_offlane, is_support) "
            f"VALUES ({hero_id}, '{name}', {is_carry}, 0, 0, {is_support});"
        )

    if missing:
        print(f"ERROR: {len(missing)} heroes have neither Carry nor Support role:")
        for name in missing:
            print(f"  - {name}")
        sys.exit(1)

    with open(OUTPUT_SQL, "w") as f:
        f.write("\n".join(lines) + "\n")

    print(f"Written {len(lines)} heroes to {OUTPUT_SQL}")


if __name__ == "__main__":
    main()
