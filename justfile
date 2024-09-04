doc:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps --open

test-local TARGET FILTER='':
    cargo test -p {{TARGET}} {{FILTER}} -- --show-output

test-all TARGET FILTER='':
    cargo test -p {{TARGET}} {{FILTER}} -- --ignored --show-output

add TARGET PACKAGE:
    cargo add -p {{TARGET}} {{PACKAGE}}

add-f TARGET PACKAGE FEATURES='':
    cargo add -p {{TARGET}} {{PACKAGE}} -F {{FEATURES}}
    
remove TARGET PACKAGE:
    cargo remove -p {{TARGET}} {{PACKAGE}}
