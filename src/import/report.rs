#[derive(Debug, Default, Clone, PartialEq)]
pub struct ImportReport {
    pub subdomains_added: usize,
    pub subdomains_skipped: usize,
    pub ips_added: usize,
    pub ips_skipped: usize,
    pub asns_added: usize,
    pub asns_skipped: usize,
    pub urls_added: usize,
    pub urls_skipped: usize,
    pub technologies_added: usize,
    pub technologies_skipped: usize,
}

impl ImportReport {
    #[allow(dead_code)]
    pub fn merge(&mut self, other: ImportReport) {
        self.subdomains_added += other.subdomains_added;
        self.subdomains_skipped += other.subdomains_skipped;
        self.ips_added += other.ips_added;
        self.ips_skipped += other.ips_skipped;
        self.asns_added += other.asns_added;
        self.asns_skipped += other.asns_skipped;
        self.urls_added += other.urls_added;
        self.urls_skipped += other.urls_skipped;
        self.technologies_added += other.technologies_added;
        self.technologies_skipped += other.technologies_skipped;
    }

    pub fn total_added(&self) -> usize {
        self.subdomains_added
            + self.ips_added
            + self.asns_added
            + self.urls_added
            + self.technologies_added
    }

    pub fn total_skipped(&self) -> usize {
        self.subdomains_skipped
            + self.ips_skipped
            + self.asns_skipped
            + self.urls_skipped
            + self.technologies_skipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_default_zeros() {
        let r = ImportReport::default();
        assert_eq!(r.total_added(), 0);
        assert_eq!(r.total_skipped(), 0);
    }

    #[test]
    fn test_report_total_added() {
        let r = ImportReport {
            subdomains_added: 2,
            ips_added: 3,
            asns_added: 1,
            urls_added: 5,
            technologies_added: 4,
            ..Default::default()
        };
        assert_eq!(r.total_added(), 15);
    }

    #[test]
    fn test_report_total_skipped() {
        let r = ImportReport {
            subdomains_skipped: 1,
            ips_skipped: 2,
            ..Default::default()
        };
        assert_eq!(r.total_skipped(), 3);
    }

    #[test]
    fn test_report_merge() {
        let mut r1 = ImportReport {
            subdomains_added: 2,
            ips_added: 1,
            ..Default::default()
        };
        let r2 = ImportReport {
            subdomains_added: 3,
            ips_skipped: 1,
            ..Default::default()
        };
        r1.merge(r2);
        assert_eq!(r1.subdomains_added, 5);
        assert_eq!(r1.ips_added, 1);
        assert_eq!(r1.ips_skipped, 1);
    }
}
