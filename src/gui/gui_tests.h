// Tests of the C++ side of the GUI. See gui_tests.cpp.

#pragma once

#include <QObject>

class GuiTests : public QObject {
    Q_OBJECT

  private slots:
    void init();
    void cleanup();

    void windowHasNoPresenceOnTheScreen();
    void clicksOnTwoPinsMakeAWire();
    void undoAndRedoFromTheMenuGoThroughEachEdit();
    void dragOfABlockIsOneUndoStep();
    void deleteKeyRemovesTheSelectionAsOneUndoStep();
    void lockFromTheContextMenuTurnsTheUndoEntryOn();
    void wiresOfOneNetHaveOneColorAndOneHighlight();
};
