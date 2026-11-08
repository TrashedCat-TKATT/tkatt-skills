// The mod's settings page: what the bar holds, how many points are waiting,
// the armour and Bulwark totals, every big skill's on/off state and rank, and
// a button to clear the trees.
fn skill_trees_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use crate::game::tkatt_skills as m;
    if mod_id != m::ID {
        return;
    }
    let skills = {
        let game = &*game;
        m::load_once(|key| game.native_setting(m::ID, key).or_else(|| game.native_setting(m::LEGACY_ID, key)))
    };
    let percent = (skills.bank / m::XP_PER_POINT * 100.0).round() as u32;
    buttons.push(format!("Bar: {percent}% of the next point"));
    buttons.push(format!("Points to spend: {}", skills.points));
    buttons.push(format!(
        "Fists: {} ({})",
        m::FIST_TIERS[skills.fist_tier() as usize],
        skills.fist_speed()
    ));
    let tune = m::tuning();
    buttons.push(format!("Armour: -{:.0}% damage", skills.armor_cut(tune) * 100.0));
    buttons.push(format!(
        "Bulwark: {}/{} ({:.0}% cut)",
        skills.levels[m::Tree::Bulwark.index()],
        m::BULWARK_MAX,
        skills.ward_cut(tune) * 100.0
    ));
    for row in [m::BIG_MINING, m::BIG_FORTUNE, m::BIG_HASTE] {
        let state = if skills.big_rank(row) == 0 {
            "locked"
        } else if skills.big_on(row) {
            "on"
        } else {
            "off"
        };
        buttons.push(format!(
            "{}: {} ({}/{})",
            m::TREES[row].title(),
            state,
            skills.big_rank(row).min(tune.max(row)),
            tune.max(row)
        ));
    }
    buttons.push("Reset trees".to_owned());
}
