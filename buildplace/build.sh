flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir com.mediamagician.app.yml

flatpak build-bundle repo MediaMagician.flatpak com.mediamagician.app
