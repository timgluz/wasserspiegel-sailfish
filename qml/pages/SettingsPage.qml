import QtQuick 2.0
import Sailfish.Silica 1.0

Page {
    id: page

    property string testResult
    property bool testOk: false

    SilicaFlickable {
        anchors.fill: parent
        contentHeight: content.height

        Column {
            id: content
            width: parent.width
            spacing: Theme.paddingLarge

            PageHeader {
                title: qsTr("Settings")
            }

            TextField {
                id: apiBaseField
                width: parent.width
                label: qsTr("API URL")
                placeholderText: "https://wasserspiegel.example.org"
                text: appController.apiBase
                inputMethodHints: Qt.ImhUrlCharactersOnly
                EnterKey.enabled: text.trimmed().length > 0
                EnterKey.iconSource: "image://theme/icon-m-enter-next"
                EnterKey.onClicked: apiTokenField.focus = true
            }

            TextField {
                id: apiTokenField
                width: parent.width
                label: qsTr("API token")
                placeholderText: qsTr("Bearer token")
                echoMode: TextInput.Password
                text: appController.apiToken
                EnterKey.enabled: text.trimmed().length > 0
                EnterKey.iconSource: "image://theme/icon-m-enter-accept"
                EnterKey.onClicked: saveButton.clicked()
            }

            Button {
                id: saveButton
                anchors.horizontalCenter: parent.horizontalCenter
                text: qsTr("Save & test")
                enabled: apiBaseField.text.trimmed().length > 0
                         && apiTokenField.text.trimmed().length > 0

                onClicked: {
                    testResult = ""
                    appController.applySettings(apiBaseField.text.trimmed(),
                                                apiTokenField.text.trimmed())
                    appController.testConnection()
                }
            }

            Label {
                visible: page.testResult !== ""
                width: parent.width - 2 * Theme.horizontalPageMargin
                x: Theme.horizontalPageMargin
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
                font.pixelSize: Theme.fontSizeSmall
                color: page.testOk ? Theme.highlightColor : Theme.errorColor
                text: page.testResult
            }

            SectionHeader {
                text: qsTr("About")
            }

            DetailItem {
                label: qsTr("Version")
                value: "0.1.0"
            }
        }
    }

    Connections {
        target: appController
        onConnectionTested: {
            page.testOk = ok
            page.testResult = message
            if (ok) {
                // credentials work - go back so the dashboard refreshes
                autoPop.start()
            }
        }
    }

    Timer {
        id: autoPop
        interval: 1500
        onTriggered: pageStack.pop()
    }
}
