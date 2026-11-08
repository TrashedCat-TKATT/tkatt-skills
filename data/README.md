# Icon data

`fist_icon.png` is the original emblem for the fist mining tier.
`fist_icon.rgba` is the 24x24 RGBA the bundle compiles in with `native_data!`,
generated from it by `tools/embed_icon.py`. Keep both: the PNG is the source,
the RGBA is what the game uploads with `native_mod_image`.

The bundle's `data/` folder is staged as `native-data/tkatt_skills/` when the
game is built, so a file here costs a compile-in and nothing at run time.
