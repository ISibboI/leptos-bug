use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use leptos::prelude::*;
use log::Level;
use reactive_stores::{Field, Store, StoreFieldIterator};

#[derive(Debug, Store)]
pub struct GlobalState {
    population: Population,
}

impl GlobalState {
    pub fn new_debug_instance() -> Self {
        Self {
            population: Population::new(),
        }
    }

    pub fn finish_week(this: Store<Self>) {
        Population::finish_week(this.population().into());
    }
}

fn main() {
    // Get better error messages from WASM in the browser.
    console_error_panic_hook::set_once();
    // Init logging in the browser console.
    console_log::init_with_level(Level::Debug).unwrap();

    mount_to_body(App)
}

#[component]
fn App() -> impl IntoView {
    provide_context(Store::new(GlobalState::new_debug_instance()));
    let state = expect_context::<Store<GlobalState>>();

    view! {
        <button on:click=move |_| GlobalState::finish_week(state)>Finish Week</button>
        <PersonView />
    }
}

#[derive(Debug, Default, Store)]
pub struct Population {
    #[store(skip)]
    people_by_id: HashMap<PersonId, usize>,
    #[store(key: PersonId = |row| row.key())]
    people: Vec<Person>,
}

impl Population {
    pub fn new() -> Self {
        let mut people = Vec::new();
        for _ in 0..5 {
            people.push(Person::create());
        }

        Self {
            people_by_id: people
                .iter()
                .enumerate()
                .map(|(index, person)| (person.key(), index))
                .collect(),
            people,
        }
    }

    pub fn person(this: Field<Self>, person_id: PersonId) -> Field<Person> {
        let index = *this
            .read()
            .people_by_id
            .get(&person_id)
            .unwrap_or_else(|| panic!("Did not find person {:?} in index", person_id));
        this.people().iter_unkeyed().nth(index).unwrap().into()
    }

    pub fn finish_week(this: Field<Self>) {
        // Finish weeks for people.
        for person in this.people().iter_unkeyed() {
            Person::finish_week(person.into());
        }
    }
}

static NEXT_PERSON_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Store)]
pub struct Person {
    id: PersonId,
    happiness: Happiness,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PersonId(u64);

impl Person {
    pub fn create() -> Self {
        Self {
            id: PersonId(NEXT_PERSON_ID.fetch_add(1, Ordering::Relaxed)),
            happiness: Happiness::new_initial(),
        }
    }

    pub fn key(&self) -> PersonId {
        self.id
    }

    pub fn finish_week(this: Field<Self>) {
        Happiness::finish_week(this);
    }
}

#[component]
pub fn PersonView() -> impl IntoView {
    let person_id = PersonId(2);

    view! { <HappinessModifierTable person_id=person_id /> }
}

#[derive(Debug, Store)]
pub struct Happiness {
    #[store(key: HappinessModifierId = |row| row.key())]
    happiness_modifiers: Vec<HappinessModifier>,
}

impl Happiness {
    pub fn new_initial() -> Self {
        Self {
            happiness_modifiers: vec![HappinessModifier::create()],
        }
    }

    pub fn finish_week(person: Field<Person>) {
        // Reset happiness modifiers. These are recomputed every week.
        person.happiness().happiness_modifiers().write().clear();
        Self::add_happiness_modifier(person.happiness().into());
    }

    pub fn add_happiness_modifier(this: Field<Self>) {
        this.happiness_modifiers()
            .write()
            .push(HappinessModifier::create())
    }
}

#[component]
pub fn HappinessModifierTable(#[prop(into)] person_id: Signal<PersonId>) -> impl IntoView {
    let state = expect_context::<Store<GlobalState>>();
    let person = Population::person(state.population().into(), person_id.get());
    let happiness = person.happiness();

    view! {
        <For
            each=move || { happiness.happiness_modifiers() }
            key=|row| row.read().key()
            children=move |child| {
                view! { <HappinessModifierTableEntry happiness_modifier=child /> }
            }
        />
    }
}

static NEXT_HAPPINESS_MODIFIER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Store)]
pub struct HappinessModifier {
    id: HappinessModifierId,
    kind: HappinessModifierKind,
}

#[derive(Debug, Clone, Copy)]
pub enum HappinessModifierKind {
    Default,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct HappinessModifierId(u64);

impl HappinessModifier {
    pub fn create() -> Self {
        Self {
            id: HappinessModifierId(NEXT_HAPPINESS_MODIFIER_ID.fetch_add(1, Ordering::Relaxed)),
            kind: HappinessModifierKind::Default,
        }
    }

    pub fn key(&self) -> HappinessModifierId {
        self.id
    }

    pub fn happiness(this: Field<Self>) -> f64 {
        this.kind().try_read().unwrap().happiness()
    }
}

impl HappinessModifierKind {
    pub fn happiness(&self) -> f64 {
        match self {
            Self::Default => 0.5,
        }
    }
}

#[component]
pub fn HappinessModifierTableEntry(
    #[prop(into)] happiness_modifier: Field<HappinessModifier>,
) -> impl IntoView {
    let happiness = Signal::derive(move || HappinessModifier::happiness(happiness_modifier));

    view! { {move || format!("{:.0}%", happiness.get() * 100.0)} }
}
