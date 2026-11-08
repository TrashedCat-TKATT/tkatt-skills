// The bar in its corner, with the tree panel under it while open. The origin
// is kept so the key hook can hit-test the player's click against the same
// pixels.
fn skill_trees_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::tkatt_skills as m;
    // The effect hooks have no `Game` to ask, so the frame records it here.
    m::set_enabled(game.native_on(m::ID));
    // Turned off in Options > Mods: nothing until the next build leaves it out.
    if !game.native_on(m::ID) {
        return;
    }
    // Renamed from `skill_trees`: carry its two files over once, so no banked
    // experience or tuning is lost. The loaders read the old id until then.
    if !m::carried() {
        let carried = m::carry(
            game.native_setting(m::LEGACY_ID, "state"),
            game.native_setting(m::LEGACY_ID, "tuning"),
            game.native_setting(m::ID, "state"),
            game.native_setting(m::ID, "tuning"),
        );
        for (key, value) in carried {
            game.native_set_setting(m::ID, key, &value);
        }
        m::set_carried();
    }
    use horizon_client::extras::Corner;
    let corner = Corner::BottomLeft;
    if !game.native_corner_shown(corner) {
        return;
    }
    let skills = {
        let game = &*game;
        m::load_once(|key| game.native_setting(m::ID, key).or_else(|| game.native_setting(m::LEGACY_ID, key)))
    };
    let (open, holding) = m::ui();
    let view = m::View::of(&skills, open, holding);
    let (w, h) = m::size(&view);
    let (x, y) = corners.place(corner, w, h);
    m::set_origin((x, y));
    let game = &*game;
    for shape in m::shapes(&view) {
        match shape {
            m::Shape::Fill([dx, dy, sw, sh], argb) => {
                game.native_fill(batch, [x + dx, y + dy, sw, sh], argb);
            }
            m::Shape::Text(text, [dx, dy], rgb, shadow) => {
                game.native_text(batch, &text, [x + dx, y + dy], rgb, shadow);
            }
        }
    }
}
