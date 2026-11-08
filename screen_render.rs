// The open skill screen draws itself over the dimmed world: the panel, its
// tabs, its nodes (the fist's is the mod's own emblem) and the footer. The
// screen's hooks are `&self`, so this reads and draws only.
fn skill_trees_screen_render(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use crate::game::tkatt_skills as m;
    if screen.id != "mod" || game.native_mod_screen() != Some(m::ID) {
        return;
    }
    let at = m::panel_origin(screen.rect);
    let local = [screen.mouse[0] - at[0], screen.mouse[1] - at[1]];
    let skills = m::load_once(|key| {
        game.native_setting(m::ID, key)
            .or_else(|| game.native_setting(m::LEGACY_ID, key))
    });
    let view = m::ScreenView::of(&skills, m::screen(), local);
    // A screen has no backdrop of its own: dim the world behind it here.
    game.native_fill(
        batch,
        [0.0, 0.0, screen.rect[2], screen.rect[3]],
        0x80_00_00_00,
    );
    for shape in m::screen_shapes(&view, m::icon_ready()) {
        match shape {
            m::ScreenShape::Fill(rect, argb) => {
                game.native_fill(
                    batch,
                    [at[0] + rect[0], at[1] + rect[1], rect[2], rect[3]],
                    argb,
                );
            }
            m::ScreenShape::Text(text, [dx, dy], rgb, shadow) => {
                game.native_text(batch, &text, [at[0] + dx, at[1] + dy], rgb, shadow);
            }
            m::ScreenShape::TextRight(text, x, y, rgb, shadow) => {
                let width = game.native_text_width(&text);
                game.native_text(batch, &text, [at[0] + x - width, at[1] + y], rgb, shadow);
            }
            m::ScreenShape::TextCenter(text, y, rgb, shadow) => {
                let width = game.native_text_width(&text);
                game.native_text(
                    batch,
                    &text,
                    [at[0] + (m::SCREEN_W - width) / 2.0, at[1] + y],
                    rgb,
                    shadow,
                );
            }
            m::ScreenShape::Button {
                label,
                rect,
                enabled,
                hovered,
            } => {
                let sprite = if !enabled {
                    "gui/sprites/widget/button_disabled"
                } else if hovered {
                    "gui/sprites/widget/button_highlighted"
                } else {
                    "gui/sprites/widget/button"
                };
                game.native_sprite(
                    batch,
                    sprite,
                    [0.0, 0.0, 200.0, 20.0],
                    [at[0] + rect[0], at[1] + rect[1], rect[2], rect[3]],
                );
                let width = game.native_text_width(&label);
                game.native_text(
                    batch,
                    &label,
                    [
                        at[0] + rect[0] + (rect[2] - width) / 2.0,
                        at[1] + rect[1] + (rect[3] - 9.0) / 2.0 + 1.0,
                    ],
                    if enabled { 0xFF_FF_FF_FF } else { 0xFF_A0_A0_A0 },
                    false,
                );
            }
            m::ScreenShape::Icon(item, [x, y], size) => {
                game.native_item(batch, item, [at[0] + x, at[1] + y], size);
            }
            m::ScreenShape::Emblem([x, y], size) => {
                game.native_sprite(
                    batch,
                    "horizon/modimage",
                    [0.0, 0.0, size, size],
                    [at[0] + x, at[1] + y, size, size],
                );
            }
        }
    }
    if let Some(row) = m::hovered(&view) {
        let lines: Vec<horizon_client::text::Text> = m::tooltip(&view, row)
            .into_iter()
            .map(horizon_client::text::Text::literal)
            .collect();
        game.native_gui_tooltip(batch, &lines, screen.mouse);
    }
}
