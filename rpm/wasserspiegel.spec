Name:       wasserspiegel

Summary:    Water level dashboard for PegelOnline stations
Version:    0.2
Release:    1
License:    MIT
URL:        https://github.com/timgluz/wasserspiegel-sailfish
Source0:    %{name}-%{version}.tar.bz2
Requires:   sailfishsilica-qt5 >= 0.10.9
BuildRequires:  pkgconfig(sailfishapp) >= 1.0.2
BuildRequires:  pkgconfig(Qt5Core)
BuildRequires:  pkgconfig(Qt5Qml)
BuildRequires:  pkgconfig(Qt5Quick)
BuildRequires:  pkgconfig(Qt5Concurrent)
BuildRequires:  desktop-file-utils
# NOTE: the Rust core is prebuilt by 'task engine:rust' (see README.md) -
# qmake links rust/target/aarch64-unknown-linux-gnu/release/libwasserspiegel_core.a

%description
Shows current water levels from the German PEGELONLINE service. Browse
measurement stations, follow water level trends and keep your favourite
station on the home screen cover.


%prep
%setup -q -n %{name}-%{version}

%build

%qmake5 

%make_build


%install
%qmake5_install


desktop-file-install --delete-original         --dir %{buildroot}%{_datadir}/applications                %{buildroot}%{_datadir}/applications/*.desktop

%files
%defattr(-,root,root,-)
%{_bindir}/%{name}
%{_datadir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/*/apps/%{name}.png

%changelog
* Sat Aug 15 2026 Timo Sulg <timgluz@gmail.com> - 0.2-1
- Recent stations, GPS nearest station, logs page, about page
- Demo data fallback and config-flow improvements
* Sat Aug 15 2026 Timo Sulg <timgluz@gmail.com> - 0.1-1
- Initial release
