# Oversight

This is an attempt to simulate combats between a player character and an enemy using [Stoneshard](https://stoneshard.com/)'s rules written in [Rust](https://www.rust-lang.org/). The main purpose of this project is to determine if a setup of stats/skills is enough to defeat an specific foe. 

Currently, the project is limited. It tries to resolve combat as a static one, only relying on basic attacks. It uses:

- damage from basic attack
- physical type of damage + poison and caustic
- accuracy & dodge and interaction between the two of them
- fumble, counter, crit, crit efficiency 
- damage modifier as main hand efficiency and weapon damage
- protection, resistance and armor penetration
- block, block depletion and recovery
- secondary effects from crit and basic attack (bleeding, daze ...), resistance and fortitude
- rules for magic damage component on protection and block
- passive skills with no complex logic increasing stats (disengage for instance)
- active skills used as buf only (warcry for instance)

It doesn't use (planned for the future):

- use of active skills during the battle (buf skills & damaged ones)
- complex logic for skills (for instance fencer stance is set at 1 stack and can't increase)
- magic damage other than caustic and poison
- energy management
- complex logic for secondary effects (for instance daze removing the use of abilities and evolving in stun if re applied)
- bodypart damage, injuries and bleeding from injuries
- pain

# Why should I use it ?

Using player knowledge about combat, this tool may help to choose between two differents passives to select following a level-up. For instance, if you struggle surviving combat and hesistate between two passives, you could try the two different setups against a Bear and decide which one improve your chance of success.

# Why should I NOT use it ?

The tool is not made to decide which skills are the best nor how you can optimize your playstyle. Currently not all skills are supported, and there is no intention to add an algorithm to find some optimal patterns.

# Uses

The project uses currently three differents yaml datafiles:

- [action.yaml](data/action.yaml) describing the skills applyed on the player character and the enemy.
- [characters.yaml](data/characters.yaml) storing data about the stats of the player character and diverse foe. Data come from either the wiki or knowledge from the ingame tooltip.
- [effects.yaml](data/effects.yaml) describing the effect skills have on stats.

You can edit your specific stats of your character in the characters.yaml file under the "main" field and add more foes (Moose/Gulon, ...). You can also change the action.yaml file to specify how you and your foe will start the combat. Your effects are under the "self" field, and your foe ones under the "other" field. Supported skills are listed in the "IdSkills" enum in [hit.rs](src/hit.rs). 

Adding more effects and skills in the effects.yaml won't be enough to use them in the action.yaml (it will also crash the code).

To run the code, you need to have rust and cargo installed and then just run the following: 
```
cargo run
```

# Disclaimer

It's a toy project, so there is no guarantee the combat system implemented is 100% correct.