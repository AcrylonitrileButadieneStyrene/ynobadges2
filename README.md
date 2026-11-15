# ynobadges2

High level badge definition format for simpler badge implementation and 
management

Badges are located at `badges/`**`batch`**`/`**`game`**`/`**`badge_id`**`.toml`

The batch is stored in the file system path, so changing the batch for many
badges at once is as simple as moving them from one folder to another. All badge
data is located in a single file each, so renaming the file does not break it*.

<sub>\*badge art is not managed by ynobadges2</sub>

## Implementing a badge

Each badge has 4 components: a badge file with the file name determining the 
id, a set of condition files linked to from the main badge file through 
reqStrings, entries in each of the localization files in the `lang/` folder, and
the badge art in the `images/` folder. The key in the localization and the file
name of the badge art is determined by the badge id.

All of those components except for the badge art can be created by ynobadges2's
compiler from a single file, with certain implementation details like request 
tags (linking condition files to the badge) being automatically managed,
reducing the complexity of badge implementation*.

<sub>*even if implementation is not being done by hand, a simpler format makes
for simpler code in programs generating badges</sub>

## Badge format

Badges are defined in TOML, an extended INI format.

The available options are listed in the `schema.json` file.

There are 3 top level sections: 
- `[badge]`: metadata about the badge itself such as the map (for sorting
and location text on the website), the points it is for, if the badge is 
animated (a .gif file will be expected), who the artist is, the group the badge
is sorted under, and more.
- `[conditions]`: A map between condition IDs and the condition code (see 
below), with two
special values:
    - `default`: changed into the badge id by the compiler
    - `requirements`: a list of requirements (see below). If not specified, 
    defaults to requiring all conditions defined in the section.
- `[lang]`:

## Requirements string format

A list of condition IDs separated by a logical separator (`AND` or `OR`).

Exmaples:
- `a OR b` (either `a` or `b` for the badge to be unlocked)
- `a AND b` (both `a` or `b` for the badge to be unlocked)
  - if both `a` and `b` were defined directly in the badge file, this would be
  the default value if the `requirements` are not specified.
- `a OR b AND c` (requires either `a` or both `b` and `c`)
- `a AND b OR c` (requires either `a` and `b` or only `c`)

This format is capable of expressing all possible forms that are supported

## Conditions string format

A simple list of conditions which all must be true for the condition to unlock.

Examples:
- `M100` (be on map 100)
- `M200 E1` (be on map 200 and interact with event 1)
- `M300 X=0:50` (be on map 300 with an x value between 0 and 50)

Full list of commands:
- `M[id]`: be on map `id`
- `E[id]`: interact with event `id`
- `S[id]={ON,OFF}`: switch `id` is either on or off
- `V[id][condition][value]`: variable `id` compared with `value`
- `{X,Y}=[min]:[max]`: player's x or y value between `min` and `max`
- `Picture='[value]'`: when a picture event named `value` is on screen
- `DELAYED`: after a `S` or `V` command, sets `varDelay` or `switchDelay`
- `INDIRECT`: changes the `trigger` from `event action` to `event`
  - (i do not understand when this is meant to be used)
- `TimeTrial`: see `badges/229/2kki/tt_waters_edge.toml` for usage
