// The mod in Options > Mods, with its panel key.
fn skill_trees_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::tkatt_skills as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.to_owned(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.to_owned(),
        keys: vec![[
            "panel".to_owned(),
            "Open skill trees".to_owned(),
            "key.keyboard.k".to_owned(),
        ]],
    });
}
