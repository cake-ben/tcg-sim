use crate::card::{Card, CardType, CardFragmentKind, CreatureFragment, CreatureStats};

pub fn is_creature(card: &Card) -> bool
{
    card.card_types.iter().any(|ct| *ct == CardType::Creature)
        || card.fragments.contains_key(&CardFragmentKind::Creature)
}

pub fn creature_stats(card: &Card) -> Option<CreatureStats>
{
    card.fragments.get(&CardFragmentKind::Creature).and_then(|f|
        f.as_any().downcast_ref::<CreatureFragment>().map(|cf| cf.stats)
    )
}

pub fn add_creature_fragment(card: &mut Card, power: u8, toughness: u8)
{
    card.fragments.insert(
        CardFragmentKind::Creature,
        Box::new(CreatureFragment { stats: CreatureStats { power, toughness }, summoning_sickness: false }),
    );
}

pub fn remove_creature_fragment(card: &mut Card)
{
    card.fragments.remove(&CardFragmentKind::Creature);
}

pub fn set_summoning_sickness(card: &mut Card, value: bool)
{
    if let Some(f) = card.fragments.get_mut(&CardFragmentKind::Creature)
    {
        if let Some(cf) = f.as_any_mut().downcast_mut::<CreatureFragment>()
        {
            cf.summoning_sickness = value;
        }
    }
}

pub fn has_summoning_sickness(card: &Card) -> bool
{
    card.fragments.get(&CardFragmentKind::Creature)
        .and_then(|f| f.as_any().downcast_ref::<CreatureFragment>().map(|cf| cf.summoning_sickness))
        .unwrap_or(false)
}
