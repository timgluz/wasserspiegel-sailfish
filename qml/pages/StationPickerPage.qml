import QtQuick 2.0
import Sailfish.Silica 1.0
import QtPositioning 5.0

Page {
    id: page

    property bool locating: false
    property string locateError: ""

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
                placeholderText: qsTr("e.g. Mannheim or Rhein")
                onTextChanged: appController.searchStations(text)

                EnterKey.enabled: text.length > 0
                EnterKey.iconSource: "image://theme/icon-m-search"
                EnterKey.onClicked: appController.searchStations(text)
            }

            Button {
                anchors.horizontalCenter: parent.horizontalCenter
                text: page.locating ? qsTr("Locating...") : qsTr("Find nearest station")
                enabled: !page.locating && !appController.stationsLoading
                onClicked: {
                    page.locateError = ""
                    page.locating = true
                    positionSource.active = true
                    locationTimeout.restart()
                }
            }

            SectionHeader {
                visible: searchField.text.length < 2 && appController.recentStations.length > 0
                text: qsTr("Recent")
            }
        }

        ViewPlaceholder {
            enabled: listView.count === 0 && !appController.stationsLoading && !page.locating
            text: searchField.text.length < 2
                  ? qsTr("Search by name or river, or use GPS")
                  : qsTr("No matching stations")
        }

        model: searchField.text.length >= 2
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
        running: appController.stationsLoading || page.locating
        size: BusyIndicatorSize.Large
    }

    Label {
        visible: (appController.error !== "" || page.locateError !== "")
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
        text: page.locateError !== "" ? page.locateError : appController.error
    }

    PositionSource {
        id: positionSource
        active: false

        onPositionChanged: {
            if (position.latitudeValid && position.longitudeValid) {
                positionSource.active = false
                locationTimeout.stop()
                appController.findNearestStation(position.coordinate.latitude,
                                                 position.coordinate.longitude)
            }
        }

        onSourceErrorChanged: {
            if (positionSource.sourceError !== PositionSource.NoError) {
                positionSource.active = false
                locationTimeout.stop()
                page.locating = false
                page.locateError = qsTr("Location error")
            }
        }
    }

    Timer {
        id: locationTimeout
        interval: 15000
        onTriggered: {
            positionSource.active = false
            page.locating = false
            page.locateError = qsTr("Could not determine location")
        }
    }

    Connections {
        target: appController
        onStationListReady: {
            if (searchField.text.length >= 2) {
                appController.searchStations(searchField.text)
            }
        }
        onNearestStationFound: {
            page.locating = false
            if (pageStack.depth > 1) pageStack.pop()
            else pageStack.replace(Qt.resolvedUrl("DashboardPage.qml"))
        }
    }
}
