f:
    taplo fmt
    cargo +nightly fmt --all -- --error-on-unformatted --unstable-features

c:
    taplo check
    cargo clippy --workspace -- -D warnings

r:
    cargo run -- --performance-overlay ./Cargo.lock Cargo.toml

# Build and install the Flatpak locally
flatpak:
    cd flatpak && flatpak-builder --user --install --force-clean build-dir io.marc.valin.yml

# Build the Flatpak and export to a local OSTree repo (flatpak/repo/)
flatpak-repo:
    cd flatpak && flatpak-builder --repo=repo --force-clean build-dir io.marc.valin.yml
    flatpak build-update-repo flatpak/repo/

# Run the locally installed Flatpak
flatpak-run:
    flatpak run io.marc.valin

# Install Flatpak SDK/runtime prerequisites (one-time setup)
flatpak-setup:
    sudo apt install -y flatpak flatpak-builder
    flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
    flatpak install -y flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08
    flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08