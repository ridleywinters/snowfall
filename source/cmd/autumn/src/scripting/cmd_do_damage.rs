use crate::actor::Actor;

pub fn cmd_do_damage(tokens: &[&str], actor: &mut Actor) -> String {
    if tokens.len() < 2 {
        return "usage: do_damage <amount>".to_string();
    }

    let Ok(amount) = tokens[1].parse::<f32>() else {
        return format!("Invalid damage amount: {}", tokens[1]);
    };

    actor.health -= amount;
    if actor.health < 0.0 {
        actor.health = 0.0;
    }

    format!(
        "Dealt {} damage to {}, health: {}/{}",
        amount, actor.actor_type, actor.health, actor.max_health
    )
}
