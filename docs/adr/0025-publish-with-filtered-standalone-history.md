# Publish with filtered standalone history

The public Puppis S1 Manager repository preserves Puppis-specific development history by filtering the private parent repository to the `puppis_s1_linux` subtree and making that subtree the new repository root. The filtered result must be audited for deleted secrets, vendor artifacts, and unrelated paths before publication. This retains useful authorship and design context at the cost of a more careful history audit; ADR 0022 is superseded.
