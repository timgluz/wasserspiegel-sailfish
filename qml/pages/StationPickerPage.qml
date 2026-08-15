import QtQuick 2.0
import Sailfish.Silica 1.0

Page {
    id: page

    property string searchText: ""

    SilicaListView {
        id: listView
        anchors.fill: parent

        PullDownMenu {
            busy: appController.stationsLoading
            MenuItem {
                text: qsTr("Reload station list")
                onClicked: appController.testConnection()
            }
            MenuItem {
                text: qsTr("Settings")
                onClicked: pageStack.push(Qt.resolvedUrl("SettingsPage.qml"))
            }
        }

        header: Column {
            width: parent.width
            spacing: Theme.paddingMedium

            PageHeader {
                title: qsTr("Select station")
            }

            SearchField {
                id: searchField
                width: parent.width
                placeholderText: qsTr("City name or river, e.g. Mannheim")
                onTextChanged: {
                    page.searchText = text
                    appController.searchStations(text)
                }

                EnterKey.enabled: text.length > 0
                EnterKey.iconSource: "image://theme/icon-m-search"
                EnterKey.onClicked: appController.searchStations(text)
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                text: appController.gpsLocating ? qsTr("Locating...") : qsTr("Find nearest station (GPS)")
                enabled: !appController.gpsLocating && !appController.stationsLoading
                onClicked: appController.startGpsLookup()
            }

            SectionHeader {
                visible: page.searchText.length < 2 && appController.recentStations.length > 0
                text: qsTr("Recent")
            }
        }

        ViewPlaceholder {
            enabled: listView.count === 0 && !appController.stationsLoading && !appController.gpsLocating
            text: page.searchText.length < 2
                  ? qsTr("Search by city name or river, or use the GPS button above")
                  : qsTr("No matching stations")
        }

        model: page.searchText.length >= 2
               ? appController.searchResults
               : appController.recentStations

        delegate: ListItem {
            id: delegate
            contentHeight: Theme.itemSizeMedium

            Label {
                anchors {
                    left: parent.left
                    leftMargin: Theme.horizontalPageMargin
                    right: parent.right
                    rightMargin: Theme.horizontalPageMargin
                    verticalCenter: parent.verticalCenter
                }
                text: modelData.name + " / " + modelData.water
                color: delegate.highlighted ? Theme.highlightColor : Theme.primaryColor
                truncationMode: TruncationMode.Fade
            }

            onClicked: {
                appController.selectStation(modelData.id, modelData.name, modelData.water)
                if (pageStack.depth > 1) pageStack.pop()
                else pageStack.replace(Qt.resolvedUrl("DashboardPage.qml"))
            }
        }

        VerticalScrollDecorator {}
    }

    BusyIndicator {
        anchors.centerIn: parent
        running: appController.stationsLoading || appController.gpsLocating
        size: BusyIndicatorSize.Large
    }

    Label {
        visible: (appController.error !== "" || appController.gpsError !== "")
                 && !appController.stationsLoading
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: Theme.horizontalPageMargin
        }
        wrapMode: Text.Wrap
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.errorColor
        text: appController.gpsError !== "" ? appController.gpsError : appController.error
    }

    Connections {
        target: appController
        onStationListReady: {
            if (page.searchText.length >= 2) {
                appController.searchStations(page.searchText)
            }
        }
        onNearestStationFound: {
            if (pageStack.depth > 1) pageStack.pop()
            else pageStack.replace(Qt.resolvedUrl("DashboardPage.qml"))
        }
    }
}
