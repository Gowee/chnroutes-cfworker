use isocountry::CountryCode;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Registry {
    All,
    AFRINIC,
    APNIC,
    ARIN,
    LACNIC,
    RIPE,
}

impl Registry {
    pub fn stats_url(self) -> &'static str {
        match self {
            AFRINIC => "http://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-latest",
            APNIC => "http://ftp.apnic.net/stats/apnic/delegated-apnic-latest",
            ARIN => "http://ftp.arin.net/pub/stats/arin/delegated-arin-extended-latest",
            LACNIC => "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-latest",
            RIPE => "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-latest",
            All => unimplemented!(),
        }
    }

    pub fn is_all(self) -> bool {
        match self {
            All => true,
            _ => false,
        }
    }
}

impl TryFrom<&str> for Registry {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        // FIX: heap allocation
        Ok(match s.to_uppercase().as_str() {
            "AFRINIC" => Registry::AFRINIC,
            "APNIC" => Registry::APNIC,
            "ARIN" => Registry::ARIN,
            "LACNIC" => Registry::LACNIC,
            "RIPE" => Registry::RIPE,
            "All" => Registry::All,
            _ => return Err(()),
        })
    }
}
