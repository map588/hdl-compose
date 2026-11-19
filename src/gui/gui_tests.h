// Tests of the C++ side of the GUI. See gui_tests.cpp.

#pragma once

#include <QObject>

class GuiTests : public QObject {
    Q_OBJECT

  private slots:
    void init();
    void cleanup();

    void windowHasNoPresenceOnTheScreen();
    void settingsOfTheWindowAreNotTheSettingsOfTheUser();
    void clicksOnTwoPinsMakeAWire();
    void undoAndRedoFromTheMenuGoThroughEachEdit();
    void dragOfABlockIsOneUndoStep();
    void dragMovesOnlyTheBlockAndTheReleasePlacesIt();
    void pressWithASmallMoveIsAClickAndLeavesTheBlockInItsPlace();
    void dragOnTheRowOfAPortGroupMovesTheBlockWithThePointer();
    void dragOfASelectionMovesEachSelectedBlockWithThePointer();
    void undoKeepsTheViewOfTheCanvas();
    void zoomToFitPutsTheDesignInTheMiddleWithThePointerOnTheCanvas();
    void questionAboutARecoveryFileShowsTheDesignBehindIt();
    void deleteKeyRemovesTheSelectionAsOneUndoStep();
    void lockFromTheContextMenuTurnsTheUndoEntryOn();
    void wiresOfOneNetHaveOneColorAndOneHighlight();
    void changedSourceThatComesInWithAnEditShowsItsNewPins();
    void autosaveWritesTheRecoveryFileAndSaveRemovesIt();
    void saveKeySavesTheProject();
    void closeAsksAndDiscardRemovesTheRecoveryFile();
    void closeWithSaveWritesTheProject();
    void openAfterAStopOffersTheChanges_data();
    void openAfterAStopOffersTheChanges();
    void newDesignGoesIntoTheFolderOfTheOpenDesignWithNoFileDialog();
    void clickInTheListOfDesignsOpensThatDesign();
    void generatedDesignIsInTheLibraryOfANewDesignAndADoubleClickPlacesIt();
    void contextMenuRemovesAConnectionToAPortThatIsGone();
    void dropOfAModuleFromTheLibraryPlacesABlockInTheColumnOfThePointer();
    void logicMenuPlacesAGate();
    void rightClickOnTheEmptyCanvasPlacesAGateAtThePointer();
    void eachGateHasASymbolOfItsOwn();
};
