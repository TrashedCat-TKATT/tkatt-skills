// /skills shows the trees; /skills reset clears them.
// /skills points <n>, /skills level <row> <n> and /skills tune are testing and
// tuning commands, so a tree can be tried and its numbers changed without a
// build.
fn skill_trees_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    use crate::game::tkatt_skills as m;
    let mut words = command.split_whitespace();
    if words.next() != Some("skills") {
        return;
    }
    *ran = true;
    let args: Vec<&str> = words.collect();
    let op = match m::op(&args) {
        Ok(op) => op,
        Err(usage) => {
            game.native_chat(&usage);
            return;
        }
    };
    match op {
        m::Op::TuneShow => {
            game.native_chat(&m::tuning().show());
            return;
        }
        m::Op::Tune(tree, field, value) => {
            let mut tune = m::tuning();
            if let Err(reply) = tune.set(tree, field, value) {
                game.native_chat(&reply);
                return;
            }
            m::store_tune(tune);
            game.native_set_setting(m::ID, "tuning", &tune.encode());
            game.native_chat(&format!(
                "tkatt-skills: {} {} is now {}; {}",
                tree.title(),
                field.name(),
                value,
                tune.show()
            ));
            return;
        }
        _ => {}
    }
    let mut skills = m::load_once(|key| game.native_setting(m::ID, key).or_else(|| game.native_setting(m::LEGACY_ID, key)));
    if !op.changes() {
        game.native_chat(&m::status(&skills));
        return;
    }
    skills.run(op);
    m::store(skills.clone());
    game.native_set_setting(m::ID, "state", &skills.encode());
    game.native_chat(&skills.done(op));
}
