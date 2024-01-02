#[derive(Debug, Copy, Clone)]
pub enum Config {
    BrittleEffect,
    Boss,
    CoveredInAsh,
    CustomMods,
    EnemyBrittled,
    EnemeyHit,
    EnemySapped,
    EnemyScorched,
    EnemyShocked,
    Focused,
    FrenzyCharges,
    FrenzyChargesAmount,
    PowerCharges,
    PowerChargesAmount,
    WitherStacks,
    SapEffect,
    ScorchedEffect,
    ShockEffect,
}

impl Config {
    fn name(&self) -> &'static str {
        match self {
            Self::BrittleEffect => "conditionBrittleEffect",
            Self::Boss => "enemyIsBoss",
            Self::CoveredInAsh => "conditionEnemyCoveredInAsh",
            Self::CustomMods => "customMods",
            Self::EnemyBrittled => "conditionEnemyBrittle",
            Self::EnemeyHit => "enemyHit",
            Self::EnemySapped => "conditionEnemySapped",
            Self::EnemyScorched => "conditionEnemyScorched",
            Self::EnemyShocked => "conditionEnemyShocked",
            Self::Focused => "conditionFocused",
            Self::FrenzyCharges => "useFrenzyCharges",
            Self::FrenzyChargesAmount => "overrideFrenzyCharges",
            Self::PowerCharges => "usePowerCharges",
            Self::PowerChargesAmount => "overrideFrenzyCharges",
            Self::WitherStacks => "multiplierWitheredStackCount",
            Self::SapEffect => "conditionSapEffect",
            Self::ScorchedEffect => "conditionScorchedEffect",
            Self::ShockEffect => "conditionShockEffect",
        }
    }
}

impl From<Config> for &'static str {
    fn from(stat: Config) -> Self {
        stat.name()
    }
}

impl PartialEq<str> for Config {
    fn eq(&self, other: &str) -> bool {
        self.name() == other
    }
}

impl PartialEq<Config> for &str {
    fn eq(&self, other: &Config) -> bool {
        &other == self
    }
}

impl PartialEq<String> for Config {
    fn eq(&self, other: &String) -> bool {
        self.name() == other
    }
}

impl PartialEq<Config> for String {
    fn eq(&self, other: &Config) -> bool {
        other == self
    }
}

pub enum ConfigValue<'a> {
    String(&'a str),
    Number(f32),
    Bool(bool),
    None,
}

impl<'a> ConfigValue<'a> {
    pub fn is_true(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            _ => false,
        }
    }

    pub fn string(&self) -> Option<&'a str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<f32> {
        match self {
            Self::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}
