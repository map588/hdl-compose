// Tests of the C++ side of the GUI.
//
// The tests make the real main window and use it the way a user does: with
// mouse clicks and key presses from Qt Test, and through the actions of the
// menus. The application runs with the Qt `offscreen` platform. That
// platform draws into memory: no window shows on the screen, and the tests
// do not use the keyboard or the mouse of the user.
//
// Each test gets a new window and a new project in a temporary directory
// (`init`). The project has the modules `source` (output `q`) and `sink`
// (inputs `d` and `e`).

// Qt Test gives its helpers for widgets and for mouse and key events only
// when these two names are defined. The build systems of Qt define them for
// a target that links Qt Widgets; this build does not use one of those.
#define QT_GUI_LIB
#define QT_WIDGETS_LIB

#include "gui_tests.h"

#include "canvas.h"
#include "items.h"

#include <QAbstractButton>
#include <QAction>
#include <QApplication>
#include <QContextMenuEvent>
#include <QDateTime>
#include <QDialog>
#include <QFile>
#include <QFileDialog>
#include <QFileSystemWatcher>
#include <QGraphicsView>
#include <QGuiApplication>
#include <QLineEdit>
#include <QListView>
#include <QListWidget>
#include <QMainWindow>
#include <QMenu>
#include <QMessageBox>
#include <QSettings>
#include <QTemporaryDir>
#include <QTest>
#include <QTimer>

#include <memory>

// Defined in app.cpp.
QMainWindow *hdlc_test_new_window();
void hdlc_test_open_project(QMainWindow *window, const QString &path);

using namespace hdlc;

namespace {

// --- The window under test ---------------------------------------------------

struct Fixture {
    QTemporaryDir dir;
    std::unique_ptr<QMainWindow> window;
    AppState *state = nullptr;
    CanvasView *view = nullptr;
    QString project;

    QString file(const QString &name) const { return dir.filePath(name); }

    void writeModule(const QString &name, const QString &ports) const {
        QFile f(file(name + QStringLiteral(".vhd")));
        QVERIFY(f.open(QIODevice::WriteOnly | QIODevice::Truncate));
        f.write(QStringLiteral("library ieee;\nuse ieee.std_logic_1164.all;\n\n"
                               "entity %1 is\n  port (%2);\nend entity;\n\n"
                               "architecture rtl of %1 is\nbegin\nend architecture;\n")
                    .arg(name, ports)
                    .toUtf8());
    }
};

std::unique_ptr<Fixture> g;

int pinCount(const InstanceItem *block) {
    int count = 0;
    for (QGraphicsItem *child : block->childItems())
        if (dynamic_cast<PortPinItem *>(child))
            ++count;
    return count;
}

QList<InstanceItem *> blocks() {
    QList<InstanceItem *> out;
    for (QGraphicsItem *item : g->view->scene()->items())
        if (auto *block = dynamic_cast<InstanceItem *>(item))
            out << block;
    return out;
}

int blocks_on_canvas() {
    return static_cast<int>(blocks().size());
}

InstanceItem *block(const QString &name) {
    for (InstanceItem *b : blocks())
        if (b->instanceName() == name)
            return b;
    return nullptr;
}

QList<WireItem *> wires() {
    QList<WireItem *> out;
    for (QGraphicsItem *item : g->view->scene()->items())
        if (auto *wire = dynamic_cast<WireItem *>(item))
            out << wire;
    return out;
}

WireItem *wireTo(const QString &target) {
    for (WireItem *w : wires())
        if (w->targetKey() == target)
            return w;
    return nullptr;
}

PortPinItem *pin(const QString &instance, const QString &port) {
    InstanceItem *b = block(instance);
    if (!b)
        return nullptr;
    for (QGraphicsItem *child : b->childItems())
        if (auto *p = dynamic_cast<PortPinItem *>(child); p && p->portName() == port)
            return p;
    return nullptr;
}

// The place of a scene point in the viewport of the canvas.
QPoint onCanvas(const QPointF &scene_pos) {
    return g->view->mapFromScene(scene_pos);
}

QPoint center(const QGraphicsItem *item) {
    return onCanvas(item->mapToScene(item->boundingRect().center()));
}

// The point of a pin that takes a click. The bounding rectangle of a pin
// goes across its block, so its center is not on the pin.
QPoint tip(const PortPinItem *pin) {
    return onCanvas(pin->tipScenePos());
}

// Move the pointer on the canvas with no button down. `QTest::mouseMove`
// is not used for this: with no button it calls `QCursor::setPos`, which
// sends no move event on the offscreen platform. The tests also must not
// use an interface that can move the pointer of the user.
void hover(const QPoint &pos, Qt::KeyboardModifiers modifiers = {}) {
    QWidget *viewport = g->view->viewport();
    QMouseEvent move(QEvent::MouseMove, pos, viewport->mapToGlobal(pos), Qt::NoButton, Qt::NoButton,
                     modifiers);
    QApplication::sendEvent(viewport, &move);
}

QAction *action(const QString &text) {
    for (QAction *a : g->window->findChildren<QAction *>())
        if (a->text().remove(QLatin1Char('&')) == text)
            return a;
    return nullptr;
}

// The window refreshes some of its state from the event loop.
void settle() {
    QCoreApplication::processEvents();
    QCoreApplication::processEvents();
}

// A menu or a message box has an event loop of its own, and the call that
// opens it returns only when it closes. So the answer must come from a
// timer that runs in that loop. `answer` gets the open popup or dialog and
// returns true when it gave the answer. `*answered` counts the answers.
void answerNext(std::function<bool(QWidget *)> answer, int *answered) {
    auto *timer = new QTimer(qApp);
    auto ticks = std::make_shared<int>(0);
    QObject::connect(timer, &QTimer::timeout, timer, [timer, ticks, answer, answered]() {
        QWidget *open = QApplication::activePopupWidget();
        if (!open)
            open = QApplication::activeModalWidget();
        if (open && answer(open)) {
            ++*answered;
            timer->stop();
            timer->deleteLater();
        } else if (++*ticks > 300) {
            // No answer in 3 s: close what is open, so that the test fails
            // on its count of answers and does not stop the run.
            if (open)
                open->close();
            timer->stop();
            timer->deleteLater();
        }
    });
    timer->start(10);
}

// Trigger the entry `text` of the next menu that opens.
void chooseFromNextMenu(const QString &text, int *answered) {
    answerNext(
        [text](QWidget *open) {
            auto *menu = qobject_cast<QMenu *>(open);
            if (!menu)
                return false;
            for (QAction *a : menu->actions()) {
                if (a->text().remove(QLatin1Char('&')) == text) {
                    menu->setActiveAction(a);
                    QTest::keyClick(menu, Qt::Key_Return);
                    return true;
                }
            }
            return false;
        },
        answered);
}

// Click the button with the text `text` of the next message box that opens.
void clickTextInNextMessageBox(const QString &text, int *answered) {
    answerNext(
        [text](QWidget *open) {
            auto *box = qobject_cast<QMessageBox *>(open);
            if (!box)
                return false;
            for (QAbstractButton *b : box->buttons()) {
                if (b->text().remove(QLatin1Char('&')) == text) {
                    b->click();
                    return true;
                }
            }
            return false;
        },
        answered);
}

QByteArray content(const QString &path) {
    QFile f(path);
    return f.open(QIODevice::ReadOnly) ? f.readAll() : QByteArray();
}

QString recoveryFile() {
    return g->file(QStringLiteral(".top.hdlc.autosave"));
}

// The window writes the recovery file 12 s after a change. The tests use
// a short time.
void useAShortAutosaveTime() {
    auto *timer = g->window->findChild<QTimer *>(QStringLiteral("autosave_timer"));
    QVERIFY(timer);
    QCOMPARE(timer->interval(), 12000);
    timer->setInterval(20);
}

// Click `button` of the next message box that opens.
void clickInNextMessageBox(QMessageBox::StandardButton button, int *answered) {
    answerNext(
        [button](QWidget *open) {
            auto *box = qobject_cast<QMessageBox *>(open);
            if (!box || !box->button(button))
                return false;
            box->button(button)->click();
            return true;
        },
        answered);
}

// What the dialogs of File > New did. A watcher answers each dialog that
// opens while it runs: it types `name` into a dialog with a text field,
// closes a message box and keeps its text, and counts and cancels a file
// dialog.
struct NewDialogs {
    int name_dialogs = 0;
    int file_dialogs = 0;
    QStringList messages;
};

NewDialogs newDesignFromTheMenu(const QString &name) {
    NewDialogs seen;
    QTimer watcher;
    QObject::connect(&watcher, &QTimer::timeout, &watcher, [&seen, name]() {
        QWidget *open = QApplication::activeModalWidget();
        if (auto *files = qobject_cast<QFileDialog *>(open)) {
            ++seen.file_dialogs;
            files->reject();
        } else if (auto *box = qobject_cast<QMessageBox *>(open)) {
            seen.messages << box->text();
            box->accept();
        } else if (auto *dialog = qobject_cast<QDialog *>(open)) {
            if (auto *edit = dialog->findChild<QLineEdit *>()) {
                edit->setText(name);
                ++seen.name_dialogs;
                dialog->accept();
            }
        }
    });
    watcher.start(10);
    // Returns when each dialog is closed.
    action(QStringLiteral("New..."))->trigger();
    watcher.stop();
    settle();
    return seen;
}

QListWidget *designsList() {
    return g->window->findChild<QListWidget *>(QStringLiteral("designs_list"));
}

// The names in the list of designs. The open design has the mark `*`.
QStringList designNames() {
    QStringList names;
    QListWidget *list = designsList();
    for (int i = 0; i < list->count(); ++i) {
        const QListWidgetItem *item = list->item(i);
        names << item->text() + (item->font().bold() ? QStringLiteral("*") : QString());
    }
    return names;
}

// Click the design with the file name `name` in the list of designs.
void clickDesign(const QString &name) {
    QListWidget *list = designsList();
    for (int i = 0; i < list->count(); ++i) {
        QListWidgetItem *item = list->item(i);
        if (QFileInfo(item->data(Qt::UserRole).toString()).completeBaseName() == name) {
            QTest::mouseClick(list->viewport(), Qt::LeftButton, {}, list->visualItemRect(item).center());
            return;
        }
    }
    QFAIL(qPrintable(QStringLiteral("no design %1 in the list").arg(name)));
}

QListView *libraryView() {
    return g->window->findChild<QListView *>(QStringLiteral("library_view"));
}

// The rows of the library list, as the user reads them.
QStringList libraryRows() {
    QStringList rows;
    const QAbstractItemModel *model = libraryView()->model();
    for (int i = 0; i < model->rowCount(); ++i)
        rows << model->index(i, 0).data(Qt::DisplayRole).toString();
    return rows;
}

} // namespace

// --- Each test ---------------------------------------------------------------

void GuiTests::init() {
    g = std::make_unique<Fixture>();
    QVERIFY(g->dir.isValid());
    g->writeModule(QStringLiteral("source"), QStringLiteral("q : out std_logic"));
    g->writeModule(QStringLiteral("sink"), QStringLiteral("d : in std_logic; e : in std_logic"));

    g->window.reset(hdlc_test_new_window());
    g->window->show();
    QVERIFY(QTest::qWaitForWindowExposed(g->window.get()));
    g->state = g->window->findChild<AppState *>();
    QVERIFY(g->state);
    for (QGraphicsView *v : g->window->findChildren<QGraphicsView *>())
        if (auto *canvas = dynamic_cast<CanvasView *>(v))
            g->view = canvas;
    QVERIFY(g->view);

    g->project = g->file(QStringLiteral("top.hdlc"));
    QVERIFY(g->state->new_project_at(g->project, 0));
    QVERIFY(g->state->add_library_path(g->file(QStringLiteral("source.vhd"))));
    QVERIFY(g->state->add_library_path(g->file(QStringLiteral("sink.vhd"))));
    QVERIFY(g->state->save_project());
    // Open it the way the operating system does. This makes the history
    // empty.
    hdlc_test_open_project(g->window.get(), g->project);
    settle();
    QVERIFY(!g->state->can_undo());
    QVERIFY(!g->state->getDirty());
}

void GuiTests::cleanup() {
    // To delete the window does not send a close event, so no dialog opens.
    g.reset();
}

void GuiTests::windowHasNoPresenceOnTheScreen() {
    QCOMPARE(QGuiApplication::platformName(), QStringLiteral("offscreen"));
    // Time for a look at the process from outside: the operating system
    // must not know it as an application with windows.
    if (qEnvironmentVariableIsSet("HDLC_GUI_TESTS_LINGER_MS"))
        QTest::qWait(qEnvironmentVariableIntValue("HDLC_GUI_TESTS_LINGER_MS"));
}

void GuiTests::clicksOnTwoPinsMakeAWire() {
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(640, 0)));
    PortPinItem *q = pin(QStringLiteral("source_0"), QStringLiteral("q"));
    PortPinItem *d = pin(QStringLiteral("sink_0"), QStringLiteral("d"));
    QVERIFY(q && d);
    QCOMPARE(g->state->wire_count(), 0);

    QTest::mouseClick(g->view->viewport(), Qt::LeftButton, {}, tip(q));
    QTest::mouseClick(g->view->viewport(), Qt::LeftButton, {}, tip(d));

    QCOMPARE(g->state->wire_count(), 1);
    QCOMPARE(g->state->port_map_entry(QStringLiteral("sink_0"), QStringLiteral("d")),
             QStringLiteral("source_0.q"));
    QVERIFY(wireTo(QStringLiteral("sink_0.d")));
}

// The handler of project_loaded puts the blocks on the grid. Before the
// correction that removed the redo history after each undo.
void GuiTests::undoAndRedoFromTheMenuGoThroughEachEdit() {
    QAction *undo = action(QStringLiteral("Undo"));
    QAction *redo = action(QStringLiteral("Redo"));
    QVERIFY(undo && redo);
    QVERIFY(!undo->isEnabled() && !redo->isEnabled());

    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(640, 0)));
    settle();
    QVERIFY(undo->isEnabled());
    QVERIFY(!redo->isEnabled());

    undo->trigger();
    settle();
    QCOMPARE(blocks().size(), 1);
    QVERIFY(redo->isEnabled());
    undo->trigger();
    settle();
    QCOMPARE(blocks().size(), 0);
    QVERIFY(!undo->isEnabled());

    redo->trigger();
    redo->trigger();
    settle();
    QCOMPARE(blocks().size(), 2);
    QCOMPARE(g->state->instance_count(), 2);
    QVERIFY(!redo->isEnabled());
}

void GuiTests::dragOfABlockIsOneUndoStep() {
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 400)));
    InstanceItem *b = block(QStringLiteral("source_1"));
    QVERIFY(b);
    const QPointF start = b->pos();
    const QPoint from = center(b);
    const QPoint to = onCanvas(b->mapToScene(b->boundingRect().center()) + QPointF(0, 300));

    QTest::mousePress(g->view->viewport(), Qt::LeftButton, {}, from);
    QTest::mouseMove(g->view->viewport(), (from + to) / 2);
    QTest::mouseMove(g->view->viewport(), to);
    QTest::mouseRelease(g->view->viewport(), Qt::LeftButton, {}, to);

    const int index = g->state->instance_index(QStringLiteral("source_1"));
    QVERIFY(g->state->instance_pos_y(index) > start.y() + 100);

    // One undo puts the block back. The next undo removes the block: the
    // drag made no second step.
    QVERIFY(g->state->undo());
    QCOMPARE(g->state->instance_pos_y(g->state->instance_index(QStringLiteral("source_1"))),
             start.y());
    QVERIFY(g->state->undo());
    QCOMPARE(g->state->instance_count(), 1);
}

void GuiTests::deleteKeyRemovesTheSelectionAsOneUndoStep() {
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(640, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(640, 400)));
    QVERIFY(g->state->connect_pins(QStringLiteral("source_0.q"), QStringLiteral("sink_0.d")).committed);
    QVERIFY(g->state->connect_pins(QStringLiteral("source_0.q"), QStringLiteral("sink_0.e")).committed);
    QVERIFY(!g->state->promote_port_to_top(QStringLiteral("sink_1"), QStringLiteral("d")).isEmpty());
    settle();
    QCOMPARE(g->state->wire_count(), 3);
    QCOMPARE(g->state->top_port_count(), 1);

    // Two wires, the top-level port and one block.
    QVERIFY(wireTo(QStringLiteral("sink_0.d")) && wireTo(QStringLiteral("sink_0.e")));
    wireTo(QStringLiteral("sink_0.d"))->setSelected(true);
    wireTo(QStringLiteral("sink_0.e"))->setSelected(true);
    for (QGraphicsItem *item : g->view->scene()->items())
        if (dynamic_cast<TopPortItem *>(item))
            item->setSelected(true);
    block(QStringLiteral("sink_1"))->setSelected(true);
    QCOMPARE(g->view->scene()->selectedItems().size(), 4);

    g->view->setFocus();
    QTest::keyClick(g->view, Qt::Key_Delete);
    settle();
    QCOMPARE(g->state->wire_count(), 0);
    QCOMPARE(g->state->top_port_count(), 0);
    QCOMPARE(g->state->instance_count(), 2);

    QVERIFY(g->state->undo());
    settle();
    QCOMPARE(g->state->wire_count(), 3);
    QCOMPARE(g->state->top_port_count(), 1);
    QCOMPARE(g->state->instance_count(), 3);
    QCOMPARE(wires().size(), 3);
}

// To lock a block sends none of the signals that the window used for the
// Undo entry, so the entry stayed off.
void GuiTests::lockFromTheContextMenuTurnsTheUndoEntryOn() {
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->state->save_project());
    hdlc_test_open_project(g->window.get(), g->project);
    settle();
    QAction *undo = action(QStringLiteral("Undo"));
    QVERIFY(!undo->isEnabled());
    InstanceItem *b = block(QStringLiteral("source_0"));
    QVERIFY(b);
    QVERIFY(!g->state->instance_locked(QStringLiteral("source_0")));

    int chosen = 0;
    chooseFromNextMenu(QStringLiteral("Lock Position"), &chosen);
    const QPoint at = center(b);
    QContextMenuEvent event(QContextMenuEvent::Mouse, at, g->view->viewport()->mapToGlobal(at));
    QApplication::sendEvent(g->view->viewport(), &event);
    settle();

    QCOMPARE(chosen, 1);
    QVERIFY(g->state->instance_locked(QStringLiteral("source_0")));
    QVERIFY(undo->isEnabled());
}

// A file can reference a load, so the pin that a wire references does not
// give its net. The canvas showed such a net in two colors.
void GuiTests::wiresOfOneNetHaveOneColorAndOneHighlight() {
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(640, 0)));
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(1280, 0)));
    // One net as a chain, and a second net with no driver.
    QVERIFY(g->state->set_port_map_entry(QStringLiteral("sink_0"), QStringLiteral("d"),
                                         QStringLiteral("source_0.q")));
    QVERIFY(g->state->set_port_map_entry(QStringLiteral("sink_1"), QStringLiteral("d"),
                                         QStringLiteral("sink_0.d")));
    QVERIFY(g->state->set_port_map_entry(QStringLiteral("sink_1"), QStringLiteral("e"),
                                         QStringLiteral("sink_0.e")));
    settle();
    WireItem *first = wireTo(QStringLiteral("sink_0.d"));
    WireItem *second = wireTo(QStringLiteral("sink_1.d"));
    WireItem *other = wireTo(QStringLiteral("sink_1.e"));
    QVERIFY(first && second && other);

    QCOMPARE(first->netKey(), second->netKey());
    QVERIFY(first->netKey() != other->netKey());
    QCOMPARE(first->pen().color(), second->pen().color());

    // Shift and the pointer on a pin of the net: each wire of the net
    // shows the highlight, and the wire of the other net fades.
    const qreal normal = first->pen().widthF();
    PortPinItem *p = pin(QStringLiteral("sink_1"), QStringLiteral("d"));
    QVERIFY(p);
    g->view->zoomToFit();
    hover(tip(p) + QPoint(0, 60));
    hover(tip(p), Qt::ShiftModifier);
    QVERIFY(first->pen().widthF() > normal);
    QVERIFY(second->pen().widthF() > normal);
    QCOMPARE(other->pen().widthF(), normal);
    QVERIFY(other->pen().color().alpha() < 255);

    // The pointer leaves the pin: no highlight stays.
    hover(tip(p) + QPoint(0, 200), Qt::ShiftModifier);
    QCOMPARE(first->pen().widthF(), normal);
    QCOMPARE(other->pen().color().alpha(), 255);
}

// --- A source that changes on the disk ---------------------------------------

// The file watcher normally reports a changed source, and the reload
// builds the canvas again. When the watcher misses the change, the next
// edit brings the new module in. The canvas then kept the old pins.
void GuiTests::changedSourceThatComesInWithAnEditShowsItsNewPins() {
    QVERIFY(g->view->placeModule(QStringLiteral("sink"), QPointF(0, 0)));
    QCOMPARE(pinCount(block(QStringLiteral("sink_0"))), 2);

    // The watcher is off for this change.
    auto *watcher = g->window->findChild<QFileSystemWatcher *>();
    QVERIFY(watcher);
    watcher->removePaths(watcher->files());
    g->writeModule(QStringLiteral("sink"),
                   QStringLiteral("d : in std_logic; e : in std_logic; f : in std_logic"));
    {
        // The library cache compares modification times.
        QFile f(g->file(QStringLiteral("sink.vhd")));
        QVERIFY(f.open(QIODevice::ReadWrite));
        QVERIFY(f.setFileTime(QDateTime::currentDateTime().addSecs(5), QFileDevice::FileModificationTime));
    }

    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(640, 0)));
    settle();

    QCOMPARE(blocks_on_canvas(), 2);
    QCOMPARE(pinCount(block(QStringLiteral("sink_0"))), 3);
    // The rebuild is not an edit: the history has the two blocks only.
    QVERIFY(g->state->undo());
    QVERIFY(g->state->undo());
    QCOMPARE(g->state->instance_count(), 0);
    QVERIFY(!g->state->can_undo());
}

// --- Recovery file -----------------------------------------------------------

void GuiTests::autosaveWritesTheRecoveryFileAndSaveRemovesIt() {
    useAShortAutosaveTime();
    const QByteArray saved = content(g->project);
    QVERIFY(!QFile::exists(recoveryFile()));

    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QTRY_VERIFY(QFile::exists(recoveryFile()));
    QVERIFY(content(recoveryFile()).contains("source_0"));
    // Only a save of the user changes the project file.
    QCOMPARE(content(g->project), saved);
    QVERIFY(g->state->getDirty());

    // File > Save.
    action(QStringLiteral("Save"))->trigger();
    QVERIFY(!g->state->getDirty());
    QVERIFY(!QFile::exists(recoveryFile()));
    QVERIFY(content(g->project).contains("source_0"));
    // No change after the save: the timer writes no new file.
    QTest::qWait(100);
    QVERIFY(!QFile::exists(recoveryFile()));
}

// Cmd+S on macOS, Ctrl+S elsewhere: Qt gives the two the same name.
void GuiTests::saveKeySavesTheProject() {
    useAShortAutosaveTime();
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QTRY_VERIFY(QFile::exists(recoveryFile()));
    QVERIFY(g->state->getDirty());

    g->window->activateWindow();
    g->view->setFocus();
    QTest::keyClick(g->view, Qt::Key_S, Qt::ControlModifier);

    QVERIFY(!g->state->getDirty());
    QVERIFY(content(g->project).contains("source_0"));
    QVERIFY(!QFile::exists(recoveryFile()));
}

void GuiTests::closeAsksAndDiscardRemovesTheRecoveryFile() {
    useAShortAutosaveTime();
    const QByteArray saved = content(g->project);
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QTRY_VERIFY(QFile::exists(recoveryFile()));

    // Cancel: the window stays, and the changes and the file stay.
    int answered = 0;
    clickInNextMessageBox(QMessageBox::Cancel, &answered);
    QVERIFY(!g->window->close());
    QCOMPARE(answered, 1);
    QVERIFY(g->window->isVisible());
    QVERIFY(QFile::exists(recoveryFile()));

    // Discard: the window closes with no save, and the file goes away.
    clickInNextMessageBox(QMessageBox::Discard, &answered);
    QVERIFY(g->window->close());
    QCOMPARE(answered, 2);
    QVERIFY(!g->window->isVisible());
    QVERIFY(!QFile::exists(recoveryFile()));
    QCOMPARE(content(g->project), saved);
}

void GuiTests::closeWithSaveWritesTheProject() {
    useAShortAutosaveTime();
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QTRY_VERIFY(QFile::exists(recoveryFile()));

    int answered = 0;
    clickInNextMessageBox(QMessageBox::Save, &answered);
    QVERIFY(g->window->close());
    QCOMPARE(answered, 1);
    QVERIFY(content(g->project).contains("source_0"));
    QVERIFY(!QFile::exists(recoveryFile()));
}

void GuiTests::openAfterAStopOffersTheChanges_data() {
    QTest::addColumn<QString>("button");
    QTest::addColumn<int>("blocks");
    QTest::addColumn<bool>("file_stays");
    QTest::newRow("recover") << QStringLiteral("Recover") << 1 << true;
    QTest::newRow("discard") << QStringLiteral("Discard") << 0 << false;
}

void GuiTests::openAfterAStopOffersTheChanges() {
    QFETCH(QString, button);
    QFETCH(int, blocks);
    QFETCH(bool, file_stays);

    useAShortAutosaveTime();
    QVERIFY(g->view->placeModule(QStringLiteral("source"), QPointF(0, 0)));
    QTRY_VERIFY(QFile::exists(recoveryFile()));
    // The program stops: the window goes away with no close event, as it
    // does when the process ends.
    g->window.reset();

    g->window.reset(hdlc_test_new_window());
    g->window->show();
    g->state = g->window->findChild<AppState *>();
    for (QGraphicsView *v : g->window->findChildren<QGraphicsView *>())
        if (auto *canvas = dynamic_cast<CanvasView *>(v))
            g->view = canvas;
    int answered = 0;
    clickTextInNextMessageBox(button, &answered);
    hdlc_test_open_project(g->window.get(), g->project);
    settle();

    QCOMPARE(answered, 1);
    QCOMPARE(g->state->instance_count(), blocks);
    QCOMPARE(blocks_on_canvas(), blocks);
    // Recovered changes are not saved yet; with Discard the project is as
    // the file has it.
    QCOMPARE(g->state->getDirty(), blocks > 0);
    QCOMPARE(QFile::exists(recoveryFile()), file_stays);
}

// --- Entry point -------------------------------------------------------------

// File > New with a design open asks for a name and a language only. The
// new design goes into the folder of the open design, where the two can use
// each other. Before, a file dialog asked for the place.
void GuiTests::newDesignGoesIntoTheFolderOfTheOpenDesignWithNoFileDialog() {
    NewDialogs seen = newDesignFromTheMenu(QStringLiteral("child"));
    QCOMPARE(seen.name_dialogs, 1);
    QCOMPARE(seen.file_dialogs, 0);
    QCOMPARE(seen.messages, QStringList());
    QVERIFY(QFile::exists(g->file(QStringLiteral("child.hdlc"))));
    QCOMPARE(QFileInfo(g->state->current_project_path()).fileName(), QStringLiteral("child.hdlc"));
    QCOMPARE(g->state->getProject_name(), QStringLiteral("child"));
    QCOMPARE(g->state->project_language(), 0);

    // The name of a design of the folder is refused, and the open design
    // stays.
    seen = newDesignFromTheMenu(QStringLiteral("top"));
    QCOMPARE(seen.file_dialogs, 0);
    QCOMPARE(seen.messages.size(), 1);
    QVERIFY2(seen.messages.first().contains(QStringLiteral("is there already")),
             qPrintable(seen.messages.first()));
    QCOMPARE(g->state->getProject_name(), QStringLiteral("child"));
}

// The sidebar lists the designs of the folder. A click on one opens it,
// with the question about changes that are not saved.
void GuiTests::clickInTheListOfDesignsOpensThatDesign() {
    // No design has generated HDL: each one has that note.
    const QString top = QStringLiteral("top (not generated)");
    const QString child = QStringLiteral("child (not generated)");
    QCOMPARE(designNames(), QStringList{top + QLatin1Char('*')});
    NewDialogs seen = newDesignFromTheMenu(QStringLiteral("child"));
    QCOMPARE(seen.messages, QStringList());
    QCOMPARE(designNames(), (QStringList{child + QLatin1Char('*'), top}));

    clickDesign(QStringLiteral("top"));
    QTRY_COMPARE(g->state->getProject_name(), QStringLiteral("top"));
    QTRY_COMPARE(designNames(), (QStringList{child, top + QLatin1Char('*')}));

    // A click on the open design does nothing.
    clickDesign(QStringLiteral("top"));
    settle();
    QCOMPARE(g->state->getProject_name(), QStringLiteral("top"));

    // With a change that is not saved, the switch asks. Cancel stays.
    QVERIFY(g->state->add_instance(QStringLiteral("u_a"), QStringLiteral("source")));
    int answered = 0;
    clickInNextMessageBox(QMessageBox::Cancel, &answered);
    clickDesign(QStringLiteral("child"));
    QTRY_COMPARE(answered, 1);
    settle();
    QCOMPARE(g->state->getProject_name(), QStringLiteral("top"));
    QVERIFY(g->state->getDirty());

    // Discard opens the other design.
    clickInNextMessageBox(QMessageBox::Discard, &answered);
    clickDesign(QStringLiteral("child"));
    QTRY_COMPARE(answered, 2);
    QTRY_COMPARE(g->state->getProject_name(), QStringLiteral("child"));
    QVERIFY(!g->state->getDirty());
    QTRY_COMPARE(designNames(), (QStringList{child + QLatin1Char('*'), top}));
}

// Generate makes the open design a module of the folder. A new design
// then has it in its library list with the mark "(design)", and a double
// click places it: the block has the top-level ports of the child, and the
// files of the child are in the library. One undo removes the two.
void GuiTests::generatedDesignIsInTheLibraryOfANewDesignAndADoubleClickPlacesIt() {
    QVERIFY(g->state->add_instance(QStringLiteral("u_a"), QStringLiteral("source")));
    g->state->promote_port_to_top(QStringLiteral("u_a"), QStringLiteral("q"));
    QVERIFY(g->state->save_project());
    QCOMPARE(designNames(), QStringList{QStringLiteral("top (not generated)*")});
    action(QStringLiteral("Generate HDL"))->trigger();
    QVERIFY(QFile::exists(g->file(QStringLiteral("top.vhd"))));
    QTRY_COMPARE(designNames(), QStringList{QStringLiteral("top*")});
    // A design is not a module for itself.
    QCOMPARE(libraryRows(), (QStringList{QStringLiteral("source"), QStringLiteral("sink")}));

    NewDialogs seen = newDesignFromTheMenu(QStringLiteral("parent"));
    QCOMPARE(seen.messages, QStringList());
    QTRY_COMPARE(designNames(),
                 (QStringList{QStringLiteral("parent (not generated)*"), QStringLiteral("top")}));
    QCOMPARE(libraryRows(), QStringList{QStringLiteral("top (design)")});
    QCOMPARE(g->state->library_path_count(), 0);

    QListView *library = libraryView();
    QTest::mouseDClick(library->viewport(), Qt::LeftButton, {},
                       library->visualRect(library->model()->index(0, 0)).center());
    settle();
    QCOMPARE(blocks_on_canvas(), 1);
    QCOMPARE(pinCount(blocks().first()), 1);
    QCOMPARE(g->state->library_path_count(), 3);
    QCOMPARE(libraryRows(), (QStringList{QStringLiteral("top (design)"), QStringLiteral("source"),
                                         QStringLiteral("sink")}));

    action(QStringLiteral("Undo"))->trigger();
    settle();
    QCOMPARE(blocks_on_canvas(), 0);
    QCOMPARE(g->state->library_path_count(), 0);
    QCOMPARE(libraryRows(), QStringList{QStringLiteral("top (design)")});
    QVERIFY(!g->state->can_undo());
}

extern "C" int run_gui_tests() {
    // The platform is fixed here, so that no call of the tests can open a
    // window on the screen.
    static char arg0[] = "hdlc-gui-tests";
    static char arg1[] = "-platform";
    static char arg2[] = "offscreen";
    static char *argv[] = {arg0, arg1, arg2, nullptr};
    static int argc = 3;
    QApplication app(argc, argv);
    if (QGuiApplication::platformName() != QLatin1String("offscreen")) {
        qCritical("the GUI tests run only with the offscreen platform");
        return 1;
    }

    // The window keeps its settings with QSettings. The tests must not
    // read or change the settings of the user.
    QTemporaryDir settings_dir;
    QSettings::setDefaultFormat(QSettings::IniFormat);
    QSettings::setPath(QSettings::IniFormat, QSettings::UserScope, settings_dir.path());

    // A dialog that no test answers must not stop the run for ever.
    QTimer::singleShot(120000, &app, []() { qFatal("the GUI tests did not end in 120 s"); });

    GuiTests tests;
    return QTest::qExec(&tests, QStringList{QString::fromLatin1(arg0)});
}
