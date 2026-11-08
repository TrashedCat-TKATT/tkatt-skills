# tkatt-skills — a native Horizon mod

Banks experience into a bar you hold, turns it into skill points, and spends
those points on four trees and three toggleable big skills. A framework for skill trees rather than a finished
game: add a tree in `runtime.rs` and one effect hook, and the UI, points and
persistence come with it.

MIT. Not made or endorsed by the Horizon project.

## What it does

- **The bar** sits in the bottom-left corner: `XP n%` on the left, the points
  you can spend on the right, filled as experience banks.
- **Hold left mouse on the bar** to bank experience. It takes 2 XP a tick from
  your own experience (through the vanilla `experience add` command, so levels,
  the bar and other players agree), fills the bar, and grants a **skill point**
  every 100 XP. Releasing stops it; the bank keeps what it has.
- **The skill key** (K by default, rebindable in Options → Mods) opens the tree
  panel under the bar. Click a row's **+** to spend a point, or to unlock and
toggle a big skill.
- **`/skills`** shows the trees and toggles the panel; **`/skills reset`** clears
  every tree and keeps your points.
- The state is written to the mod's own settings folder
  (`horizon-mods/.horizon-mod-data/tkatt_skills/state`), so it survives a
  restart, per game instance. The mod's id is `tkatt_skills`; it was
  `skill_trees` before the rename, and the first launch copies the old
  `state` and `tuning` into the new folder when it has none.

### The trees

| Tree | Per level | Levels | Effect |
| --- | --- | --- | --- |
| Melee damage | +8% damage | 10 (+80%) | `Game::melee_damage` amount, scaled |
| Armour | +0.5 armour points | 10 (+5) | flat points taken off every hit |
| Bulwark | +25% damage cut a rank | 3 (+75%) | after armour points, that share of the rest is cut |
| Fist mining | one pickaxe tier | 4 | bare hand mines and drops like wood/stone/iron/diamond |
| Mining speed | +50% mining speed a rank | 3 ranks | every block you break, up to +150% |
| Fortune | 25% a rank | 3 ranks | one more of the block's first drop, up to 75% |
| Haste | Haste II and up | 3 ranks | 200 ticks of Haste each break, one level a rank |

Fist tiers: level 1 mines like a wooden pickaxe (2.0 speed), 2 stone (4.0),
3 iron (6.0), 4 diamond (8.0) — on the blocks vanilla tags
`minecraft:mineable/pickaxe`, and a block's `needs_*_tool` tag is satisfied up
to your tier, so stone drops to a level-2 fist.

Armour points come off incoming damage first, never below zero; each Bulwark
rank then cuts a share of what is left, 25% by default and capped at 90% in
all. A hit of 8 with 5 armour points lands as 3, or as 2.25 with one Bulwark
rank.

The last three rows are big skills. Each has three ranks; a rank costs points
(the tuning sets how many, one by default). A plain click on the row's `+`
unlocks the first rank and then switches the skill on and off; Shift-click buys
the next rank.

Their numbers are not in the code: they live in the mod's data folder in a file
called `tuning`, next to `state`, and `/skills tune` changes them in game. The
line is:

```text
3|mining_step|fortune_chance|haste_ticks|haste_amp|mining_cost|fortune_cost|haste_cost|melee_step|armor_points|ward_step|mining_tier|fortune_tier|haste_tier
```

with defaults `3|0.50|0.25|200|1|1|1|1|0.08|0.50|0.25|3|3|3`. `melee_step` is
the damage added per level (+8% by default), `armor_points` the flat armour
added per level (0.5), `ward_step` the damage cut each Bulwark rank adds
(0.25, capped at 0.9 in all), the `*_tier` fields are each big skill's last
rank, and the costs are points a rank. An existing version-1 line still reads:
its old `armor_step` percentage is ignored, because that damage reduction now
lives on the Bulwark row; a version-2 line takes the default Bulwark cut. So `/skills tune melee damage 0.1`,
`/skills tune armour points 1.0`, `/skills tune bulwark cut 0.1`, `/skills tune fortune tier 4` and
`/skills tune haste cost 2` all work, and a hand-edited file is read at the
next launch. Values are clamped (skill steps 0..5, melee damage 0..1, armour
points 0..10, ward cut 0..1, chance 0..1, ticks 20..1200, amp 0..4, cost
1..10, tier 1..10), so a bad number cannot break a skill.

### Testing commands

Banking 100 XP a point is the real loop, so three commands skip it while trying
a tree:

- `/skills points <n>` adds `n` skill points.
- `/skills level <tree> <n>` sets one row outright, with `melee`, `armour`
  (or `armor`), `bulwark`, `fist`, `mining`, `fortune` or `haste` and clamping
  to the
  row's end: `/skills level fist 4` is a diamond fist, `/skills level armour 99`
  is level 10, and `/skills level fortune 1` unlocks rank 1 of Fortune.
- `/skills tune` shows the modifiers' numbers; `/skills tune <row> <field>
  <value>` changes one. Fields: `melee damage`, `armour points`, `bulwark cut`,
  `mining step`, `fortune chance`, `haste ticks`, `haste amp`, any big row's
  `cost` and any big row's `tier`.

`/skills reset` clears the trees and keeps the points, so a fresh try costs no
experience. A wrong word or a missing argument answers with the usage line
instead of changing anything.

## Controls

| Input | Action |
| --- | --- |
| Hold LMB on the bar | bank experience |
| K (rebindable in Options → Mods) | open/close the tree panel |
| LMB on a row's `+` | spend a point, or unlock and toggle a big skill |
| Shift + LMB on a big skill's `+` | buy its next rank |
| `/skills`, `/skills reset` | status and panel, or reset the trees |
| `/skills points <n>` | add skill points (for testing) |
| `/skills level <tree> <n>` | set a tree's level, clamped to its end (for testing) |

## How it is built

The bundle is a `horizon-mixins` source bundle pinned to one Horizon Nightly:

- `mixins/tkatt_skills/horizon-mixin.toml` — the manifest.
- `mixins/tkatt_skills/runtime.rs` — all the logic, and the mod's Cargo library,
  so `cargo test` runs it on its own.
- `mixins/tkatt_skills/*.rs` — one thin helper per hook, using the game's own
  `native_*` helpers.
- `mixins/tkatt_skills/mining.rs`, `fist_drops.rs`, `fortune.rs`, `haste.rs` —
  the mining, drop and Haste effects, applied where the game asks for them. The
  arithmetic lives in `runtime.rs` as `Skills::mining_factor`,
  `Skills::fist_drop`, `fortune_roll` and `fortune_target`, so `cargo test`
  covers it; the helpers only gather the
  game's facts and the stored `tuning`.

It uses Horizon's stable hook points by name (`hud`, `key`, `tick`, `mods`,
`settings`, `settings_click`, `command`, all API 1) plus the game-function
hooks that need the pin: `Game::melee_damage` (MODIFY_VARIABLE),
`Game::mob_hits_player` and `Game::hurt_player_by` (MODIFY_ARG on the amount),
and six hooks on the game's own mining code (the `progress` local and
`speed` store in `destroy_progress`, `correct_tool`/`harvest` and the
loot list's `drops` store in
`break_block_as_player` for Fortune, and a HEAD on `break_block_as_player` for
Haste), plus a cancellable HEAD on `Game::attack`. Fall, fire, lava and the other environmental sources
are deliberately untouched: those bodies carry `?`, so reducing them would mean
cancelling and re-applying the damage, which skips any other mod's hooks inside
those functions. Armour therefore covers the two damage paths that take a value
argument.

The `mods` hook (`list.rs`) gives the mod its own row in Options → Mods, and
that page lists the panel key as a rebindable row of the same kind the Key
Binds screen uses. K is only the default: rebind "Open skill trees" there and
`native_mod_key` reads the player's choice. The vanilla Key Binds screen is a
fixed table of the game's own mappings, so a mod's key lives on its page
rather than in that list.

Pinned in `compatibility.json`: Horizon `fdc87aec` (Nightly 2026-11-08) and
horizon-mixins `5a64af9` (0.6.0). The bundle is built for the launcher's
Nightly `fdc87ae` instance: choose it under Edit instance → Version.

### Test it

```sh
cargo test
```

### Stage and build

```sh
git clone https://github.com/horizon-rust/horizon ../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../horizon-mixins
cd ../horizon-mixins && cargo build -p horizon-mixins-cli
./target/debug/horizon-mixins plan --workspace ../horizon --mods ../../skill-trees/mixins --out /tmp/plan.json
./target/debug/horizon-mixins apply --workspace ../horizon --mods ../../skill-trees/mixins --out /tmp/skill-trees-game
```

The launcher does the rest: install the folder as a native mod, press Build
(`horizon-launcher native-build [FOLDER] --trust-native-code` from a shell).
Native mods run with your user's rights, are off until you install one, and are
built in an instance of their own.

## Verified and not

Verified here: the mod's 19 unit tests (banking, points, spending, unlocking,
ranking and toggling a big skill, the effect formulas and their tuning, armour
points and the ranked Bulwark cut, the
mining factor, the fist-drop rule, Fortune's roll and target, both storage lines, the box's layout and hit-testing, `/skills`
text, the test commands' parsing and clamping, and that a click on the box is
not an attack), and
`horizon-mixins plan` resolving all 17 injections, 1 module and their bindings
against the pinned Horizon source.

Also verified: the staged, patched client compiles (`cargo check -p
horizon-client --no-default-features`) with no errors and no warnings from the
mod's code. `--no-default-features` is the server build's own profile; it drops
the audio and controller crates, which need libasound and libudev headers this
environment lacks.

Not verified here: the default-feature build, and any in-game behaviour. A game
build needs a GPU/display and Horizon's prepared resources from a real
Minecraft 26.3 copy, neither of which is here. Compilation passing does not
prove the numbers feel right: check the fist tier at each level in game.

A click that lands on the box never swings at or mines what is behind it: the
cancellable HEAD on `Game::attack` returns `Break(false)` whenever the pointer
is on the drawn box, which is the whole panel while it is open, not just the
`+` boxes. The cost is that the box's corner of the screen is not an attack
surface while playing.
