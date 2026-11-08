// A click that lands on the skill box is the box's: cancel the attack for it
// so holding the bar does not swing at, or mine, whatever is behind the HUD.
// `Break(false)` returns from `Game::attack` before it picks a target.
fn skill_trees_attack(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<bool> {
    use crate::game::tkatt_skills as m;
    if !game.native_on(m::ID) || !game.native_playing() {
        return std::ops::ControlFlow::Continue(());
    }
    let (open, holding) = m::ui();
    if m::blocks_attack(&m::current(), open, holding, m::origin(), game.cursor) {
        return std::ops::ControlFlow::Break(false);
    }
    std::ops::ControlFlow::Continue(())
}
