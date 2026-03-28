# Hero Command Picker

New dotacord command: `/heroes <hero_name>`

Discord slash command args will not allow a selection list for the full 100+ heroes,
so we should get <hero_name> arg must match the localized name, 
or the localized name without spaces, eg. `Storm Spirit` or `StormSpirit`. This should be case-insensitive.

The command will open an interactive picker(see `/admin_panel` for reference) with the hero's localized name.
There will be 2 buttons that will be set or not set(again see admin panel): `Support` and `Core`.
This will allow users to easily update heroes as theirs roles change over time.

## Implementation

To facilitate this, we are going to need to have the hero details store in the database, rather than heroes.json, but we can hydrate from there.

