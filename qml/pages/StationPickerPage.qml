import QtQuick 2.0
import Sailfish.Silica 1.0

Page {
    id: page

    property string pendingQuery

    SilicaListView {
        id: listView
        anchors.fill: parent

        PullDownMenu {
            busy: appController.stationsLoading
            MenuItem {
                text: qsTr("Reload station list")
                onClicked: appController.testConnection()
            }
        }

        header: Column {
            width: parent.width

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
                onEnterClicked: appController.searchStations(text)
            }
        }

        ViewPlaceholder {
            enabled: listView.count === 0 && !appController.stationsLoading
            text: searchField.text.length < 2
                  ? qsTr("Type at least two letters")
                  : qsTr("No matching stations")
        }

        model: appController.searchResults

        delegate: ListItem {
            id: delegate
            contentHeight: Theme.itemSizeMedium

            Label {
                id: nameLabel
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
        running: appController.stationsLoading
        size: BusyIndicatorSize.Large
    }

    Label {
        visible: appController.error !== "" && !appController.stationsLoading
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: Theme.horizontalPageMargin
        }
        wrapMode: Text.Wrap
        font.pixelSize: Theme.fontSizeExtraSmall
        color: Theme.errorColor
        text: appController.error
    }

    Connections {
        target: appController
        onStationListReady: {
            if (searchField.text.length >= 2) {
                appController.searchStations(searchField.text)
            }
        }
    }
}
