set windows-shell := ["pwsh.exe", "-c"]

docs $RUSTDOCFLAGS="--cfg docsrs":
    cargo doc --no-deps -p anon_sum_types_lib