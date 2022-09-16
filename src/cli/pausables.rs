use bitflags::bitflags;

#[derive(Debug)]
pub struct PrecompileNames(Vec<String>);

bitflags! {
    pub struct PrecompileFlags: u32 {
        const EXIT_TO_NEAR     = 0b10;
        const EXIT_TO_ETHEREUM = 0b01;
    }
}

impl PrecompileNames {
    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn join(self, sep: &str) -> String {
        self.0.join(sep)
    }
}

impl From<Vec<String>> for PrecompileNames {
    fn from(s: Vec<String>) -> Self {
        PrecompileNames(s)
    }
}

impl From<Vec<&str>> for PrecompileNames {
    fn from(s: Vec<&str>) -> Self {
        Self::from(
            s.into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>(),
        )
    }
}

impl From<PrecompileFlags> for PrecompileNames {
    fn from(flags: PrecompileFlags) -> Self {
        let mut names = Vec::new();

        if flags.contains(PrecompileFlags::EXIT_TO_ETHEREUM) {
            names.push("EXIT_TO_ETHEREUM".to_owned());
        }
        if flags.contains(PrecompileFlags::EXIT_TO_ETHEREUM) {
            names.push("EXIT_TO_NEAR".to_owned());
        }

        PrecompileNames(names)
    }
}

impl From<PrecompileNames> for PrecompileFlags {
    fn from(strings: PrecompileNames) -> Self {
        let flags = strings
            .0
            .iter()
            .fold(PrecompileFlags::empty(), |mut acc, v| {
                acc.insert(match v.as_str() {
                    "EXIT_TO_NEAR" => PrecompileFlags::EXIT_TO_NEAR,
                    "EXIT_TO_ETHEREUM" => PrecompileFlags::EXIT_TO_ETHEREUM,
                    name => panic!("Unknown precompile {}", name),
                });
                acc
            });

        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    impl PrecompileNames {
        pub fn all() -> Self {
            PrecompileNames::from(vec!["EXIT_TO_NEAR", "EXIT_TO_ETHEREUM"])
        }
    }

    #[test]
    fn test_counting_precompile_names_succeeds() {
        let precompile_names = PrecompileNames::from(vec!["foo", "bar"]);
        let actual_count = precompile_names.count();
        let expected_count = 2;

        assert_eq!(expected_count, actual_count);
    }

    #[test]
    fn test_joining_precompile_names_into_string_succeeds() {
        let precompile_names = PrecompileNames::from(vec!["foo", "bar"]);
        let actual_join = precompile_names.join(";");
        let expected_join = "foo;bar";

        assert_eq!(expected_join, actual_join);
    }

    #[test]
    fn test_converting_flags_to_names_covers_all_names() {
        let precompile_flags = PrecompileFlags::all();
        let precompile_names = PrecompileNames::from(precompile_flags);
        let actual_names = precompile_names
            .0
            .into_iter()
            .sorted()
            .collect::<Vec<String>>();
        let expected_names = PrecompileNames::all()
            .0
            .into_iter()
            .sorted()
            .collect::<Vec<String>>();

        assert_eq!(expected_names, actual_names)
    }

    #[test]
    fn test_converting_names_to_flags_covers_all_flags() {
        let precompile_names = PrecompileNames::all();
        let precompile_flags = PrecompileFlags::from(precompile_names);

        assert!(
            precompile_flags.is_all(),
            "Missing flags: {:?}",
            PrecompileFlags::all().difference(precompile_flags)
        );
    }

    #[test]
    #[should_panic]
    fn test_converting_invalid_strings_to_precompile_names_panics() {
        let invalid_names = PrecompileNames::from(vec!["foo", "bar"]);

        let _ = PrecompileFlags::from(invalid_names);
    }
}
