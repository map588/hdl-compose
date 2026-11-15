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
#include <QFile>
#include <QGraphicsView>
#include <QGuiApplication>
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

QList<InstanceItem *> blocks() {
    QList<InstanceItem *> out;
    for (QGraphicsItem *item : g->view->scene()->items())
        if (auto *block = dynamic_cast<InstanceItem *>(item))
            out << block;
    return out;
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
    QVERIFY(g->state->new_project(QStringLiteral("top"), 0));
    QVERIFY(g->state->add_library_path(g->file(QStringLiteral("source.vhd"))));
    QVERIFY(g->state->add_library_path(g->file(QStringLiteral("sink.vhd"))));
    QVERIFY(g->state->save_project_as(g->project));
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

// --- Entry point -------------------------------------------------------------

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
