#include <QAction>
#include <QApplication>
#include <QCheckBox>
#include <QCloseEvent>
#include <QComboBox>
#include <QCompleter>
#include <QDialog>
#include <QDialogButtonBox>
#include <QDir>
#include <QDrag>
#include <QDragEnterEvent>
#include <QDragMoveEvent>
#include <QDropEvent>
#include <QEvent>
#include <QFile>
#include <QFileDialog>
#include <QFileInfo>
#include <QFileSystemWatcher>
#include <QFont>
#include <QFontMetrics>
#include <QFormLayout>
#include <QGraphicsEllipseItem>
#include <QGraphicsPathItem>
#include <QGraphicsRectItem>
#include <QGraphicsScene>
#include <QGraphicsSceneContextMenuEvent>
#include <QGraphicsSceneMouseEvent>
#include <QGraphicsView>
#include <QGuiApplication>
#include <QHash>
#include <QInputDialog>
#include <QKeyEvent>
#include <QLabel>
#include <QLineEdit>
#include <QListView>
#include <QMainWindow>
#include <QMap>
#include <QMenu>
#include <QMenuBar>
#include <QMessageBox>
#include <QMimeData>
#include <QMouseEvent>
#include <QObject>
#include <QPainter>
#include <QPalette>
#include <QPen>
#include <QPixmap>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QRegularExpression>
#include <QScreen>
#include <QScrollArea>
#include <QScrollBar>
#include <QSet>
#include <QSettings>
#include <QShortcut>
#include <QSplitter>
#include <QStandardItemModel>
#include <QStandardPaths>
#include <QStatusBar>
#include <QStringListModel>
#include <QStyleOptionGraphicsItem>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QTextDocument>
#include <QTimer>
#include <QToolBar>
#include <QToolTip>
#include <QTreeView>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <cmath>
#include <functional>
#include <memory>

#include "hdl-compose/src/gui/bridge.cxxqt.h"

#include "canvas_constants.h"
#include "canvas.h"
#include "items.h"

namespace {

using namespace hdlc;

void apply_material_dark_theme(QApplication &app) {
    QPalette palette;
    palette.setColor(QPalette::Window, QColor(24, 26, 28));
    palette.setColor(QPalette::WindowText, QColor(220, 222, 226));
    palette.setColor(QPalette::Base, QColor(30, 32, 35));
    palette.setColor(QPalette::AlternateBase, QColor(38, 41, 45));
    palette.setColor(QPalette::ToolTipBase, QColor(46, 49, 53));
    palette.setColor(QPalette::ToolTipText, QColor(220, 222, 226));
    palette.setColor(QPalette::Text, QColor(220, 222, 226));
    palette.setColor(QPalette::Button, QColor(40, 43, 47));
    palette.setColor(QPalette::ButtonText, QColor(220, 222, 226));
    palette.setColor(QPalette::BrightText, QColor(240, 95, 80));
    palette.setColor(QPalette::Link, QColor(120, 168, 240));
    palette.setColor(QPalette::Highlight, QColor(80, 142, 224));
    palette.setColor(QPalette::HighlightedText, QColor(245, 247, 250));
    app.setPalette(palette);

    app.setStyleSheet(QStringLiteral(
        "* { font-family: 'Inter', 'SF Pro Text', 'Segoe UI', sans-serif; }"
        "QMainWindow { background: #18191c; }"
        "QToolBar { background: #1c1e22; border: 0; padding: 4px; spacing: 4px; }"
        "QToolButton { background: transparent; color: #c8cad0; padding: 5px 10px; border-radius: 4px; }"
        "QToolButton:hover { background: #2a2d32; }"
        "QToolButton:pressed { background: #353940; }"
        "QStatusBar { background: #1c1e22; color: #9aa0aa; border-top: 1px solid #2a2d32; }"
        "QMenuBar { background: #1c1e22; color: #c8cad0; }"
        "QMenuBar::item:selected { background: #2a2d32; }"
        "QMenu { background: #1f2125; color: #c8cad0; border: 1px solid #2a2d32; }"
        "QMenu::item:selected { background: #2a2d32; }"
        "QListView, QTreeView { background: #1c1e22; alternate-background-color: #1f2125;"
        " color: #c8cad0; border: 1px solid #2a2d32; selection-background-color: #324b6f;"
        " selection-color: #f5f7fa; outline: 0; }"
        "QListView::item, QTreeView::item { padding: 3px 6px; }"
        "QSplitter::handle { background: #2a2d32; }"
        "QPlainTextEdit, QTextEdit, QLineEdit { background: #1a1c20; color: #d6d8de;"
        " border: 1px solid #2a2d32; border-radius: 3px; selection-background-color: #324b6f; }"
        "QPushButton { background: #2a2d32; color: #d6d8de; border: 1px solid #34373d;"
        " padding: 4px 12px; border-radius: 4px; }"
        "QPushButton:hover { background: #34373d; }"
        "QPushButton:pressed { background: #232529; }"
        "QPushButton:default { border: 1px solid #4a73a8; }"
        "QScrollBar:vertical { background: #1c1e22; width: 10px; margin: 0; }"
        "QScrollBar::handle:vertical { background: #34373d; border-radius: 4px; min-height: 24px; }"
        "QScrollBar::handle:vertical:hover { background: #44474d; }"
        "QScrollBar:horizontal { background: #1c1e22; height: 10px; margin: 0; }"
        "QScrollBar::handle:horizontal { background: #34373d; border-radius: 4px; min-width: 24px; }"
        "QScrollBar::handle:horizontal:hover { background: #44474d; }"
        "QScrollBar::add-line, QScrollBar::sub-line { width: 0; height: 0; }"
        "QScrollBar::add-page, QScrollBar::sub-page { background: transparent; }"
        "QGraphicsView { background: #131416; border: 0; }"
        "QHeaderView::section { background: #1c1e22; color: #9aa0aa; padding: 4px;"
        " border: 0; border-right: 1px solid #2a2d32; border-bottom: 1px solid #2a2d32; }"
    ));
}

void show_state_error(QWidget *parent, AppState *state, const QString &title) {
    QString msg = state->last_error();
    if (msg.isEmpty()) {
        msg = QStringLiteral("unknown error");
    }
    QMessageBox::critical(parent, title, msg);
}

void update_window_title(QMainWindow *window, AppState *state) {
    QString project = state->getProject_name();
    if (project.isEmpty()) {
        window->setWindowTitle(QStringLiteral("HDL Compose"));
    } else {
        QString suffix = state->getDirty() ? QStringLiteral(" *") : QString();
        window->setWindowTitle(QStringLiteral("HDL Compose — %1%2").arg(project, suffix));
    }
}

QIcon make_dirty_icon() {
    QPixmap px(12, 12);
    px.fill(Qt::transparent);
    QPainter painter(&px);
    painter.setRenderHint(QPainter::Antialiasing);
    painter.setPen(Qt::NoPen);
    painter.setBrush(QColor(220, 60, 60));
    painter.drawEllipse(1, 1, 10, 10);
    return QIcon(px);
}

} // anonymous namespace
namespace hdlc {
int find_instance_index(AppState *state, const QString &name) {
    // Lookup happens Rust-side: one FFI call instead of N QString
    // conversions across the boundary.
    return state->instance_index(name);
}
} // namespace hdlc
namespace {
using namespace hdlc;

// allocate_instance_name moved to canvas.cpp (its sole caller).

// PortPinItem, BundlePinItem, InstanceItem, TopPortItem, PinSide enum,
// format_width and find_instance_index helpers all live in items.h. The
// `using namespace hdlc;` directive above brings them in by bare name.
// Out-of-line method definitions for those classes are wrapped in
// `namespace hdlc { ... }` later in this file.

// --- PortPinItem / BundlePinItem / InstanceItem out-of-line impls -----------

} // anonymous namespace

namespace {
using namespace hdlc;

// --- Library view (drag source) ---------------------------------------------

class LibraryView : public QListView {
  public:
    explicit LibraryView(QWidget *parent = nullptr) : QListView(parent) {
        setDragEnabled(true);
        setDragDropMode(QAbstractItemView::DragOnly);
        setSelectionMode(QAbstractItemView::SingleSelection);
        setEditTriggers(QAbstractItemView::NoEditTriggers);
        setTextElideMode(Qt::ElideRight);
        setSelectionBehavior(QAbstractItemView::SelectRows);
    }

  protected:
    void startDrag(Qt::DropActions supportedActions) override {
        QModelIndexList indexes = selectedIndexes();
        if (indexes.isEmpty()) {
            return;
        }
        QString module = indexes.first().data(Qt::DisplayRole).toString();
        auto *mime = new QMimeData;
        mime->setData(QString::fromLatin1(kModuleMimeType), module.toUtf8());
        mime->setText(module);
        auto *drag = new QDrag(this);
        drag->setMimeData(mime);
        drag->exec(supportedActions, Qt::CopyAction);
    }
};

// --- Sidebar model rebuild helpers ------------------------------------------

void rebuild_tree_model(QStandardItemModel *model, AppState *state, const QIcon &dirty_icon) {
    model->clear();
    if (!state->has_project()) {
        return;
    }
    auto *root = new QStandardItem(state->getProject_name());
    root->setEditable(false);
    root->setData(QStringLiteral(""), Qt::UserRole);
    model->appendRow(root);

    int count = state->instance_count();
    for (int i = 0; i < count; ++i) {
        QString name = state->instance_name(i);
        QString module = state->instance_module(i);
        auto *item = new QStandardItem(QStringLiteral("%1 : %2").arg(name, module));
        item->setEditable(false);
        item->setData(name, Qt::UserRole);
        if (state->instance_is_dirty(i)) {
            item->setIcon(dirty_icon);
        }
        root->appendRow(item);

        // Dependencies as child rows
        int dc = state->instance_dependency_count(i);
        for (int d = 0; d < dc; ++d) {
            QString dep_name = state->instance_dependency_name(i, d);
            bool present = state->instance_dependency_present(i, d);
            QString label = present ? dep_name : QStringLiteral("\u26A0 %1").arg(dep_name); // ⚠ prefix
            auto *dep_item = new QStandardItem(label);
            dep_item->setEditable(false);
            dep_item->setSelectable(false);
            // Empty UserRole — clicking a dep row shouldn't trigger selection
            dep_item->setData(QStringLiteral(""), Qt::UserRole);
            if (!present) {
                dep_item->setData(QVariant(QColor(220, 60, 60)), Qt::ForegroundRole);
                dep_item->setToolTip(QStringLiteral("Module not in library"));
            } else {
                dep_item->setToolTip(QStringLiteral("Module %1 present in library").arg(dep_name));
            }
            item->appendRow(dep_item);
        }
    }
}

void rebuild_library_model(QStringListModel *model, AppState *state) {
    QStringList modules;
    int lc = state->library_module_count();
    for (int i = 0; i < lc; ++i) {
        modules << state->library_module_name(i);
    }
    model->setStringList(modules);
}

static QString default_open_dir() {
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
    QString dir = settings.value(QStringLiteral("default_open_dir")).toString().trimmed();
    if (dir.isEmpty()) {
        dir = QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation);
    }
    return dir;
}

// --- Recent projects (QSettings-backed MRU list) -----------------------------

constexpr int kMaxRecentProjects = 8;

QStringList recent_projects() {
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
    return settings.value(QStringLiteral("recent_projects")).toStringList();
}

void add_recent_project(const QString &path) {
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
    QStringList list = settings.value(QStringLiteral("recent_projects")).toStringList();
    list.removeAll(path);
    list.prepend(path);
    while (list.size() > kMaxRecentProjects) {
        list.removeLast();
    }
    settings.setValue(QStringLiteral("recent_projects"), list);
}

static QString sh_quote(const QString &s) {
    QString escaped = s;
    escaped.replace(QStringLiteral("'"), QStringLiteral("'\\''"));
    return QStringLiteral("'") + escaped + QStringLiteral("'");
}

void launch_goto_source(QWidget *parent, const QString &source_path) {
    if (source_path.isEmpty()) {
        QMessageBox::information(parent, QStringLiteral("Goto Source"), QStringLiteral("Source path unavailable."));
        return;
    }
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
    QString cmd = settings.value(QStringLiteral("editor_command")).toString().trimmed();
    if (cmd.isEmpty()) {
        QMessageBox::information(parent, QStringLiteral("Goto Source"),
                                 QStringLiteral("No external editor configured. Set one in "
                                                "File → Preferences."));
        return;
    }
    bool in_terminal = settings.value(QStringLiteral("editor_in_terminal"), false).toBool();

    QStringList parts = cmd.split(QRegularExpression(QStringLiteral("\\s+")), Qt::SkipEmptyParts);
    QString program = parts.takeFirst();
    parts << source_path;

    if (in_terminal) {
#ifdef __APPLE__
        // .app bundles have no TTY — terminal editors (nvim, vim, emacs -nw)
        // exit immediately. Launch inside the user's preferred macOS terminal
        // app via osascript. Default "Terminal" (always present on macOS);
        // user-configurable in Preferences (e.g. "iTerm", "Alacritty",
        // "kitty", "Ghostty" — anything that implements `do script`).
        QString terminal_app = settings
            .value(QStringLiteral("terminal_app"), QStringLiteral("Terminal"))
            .toString()
            .trimmed();
        if (terminal_app.isEmpty()) {
            terminal_app = QStringLiteral("Terminal");
        }
        QString shell_cmd = sh_quote(program);
        for (const QString &p : parts) {
            shell_cmd += QChar(' ');
            shell_cmd += sh_quote(p);
        }
        QString as_escaped = shell_cmd;
        as_escaped.replace(QChar('\\'), QStringLiteral("\\\\"));
        as_escaped.replace(QChar('"'), QStringLiteral("\\\""));
        // iTerm uses a different AppleScript dialect — `do script` is a
        // Terminal.app-only command. iTerm needs `create window` + a session
        // `write text`. Branch on app name so both Just Work.
        QString script;
        if (terminal_app.compare(QStringLiteral("iTerm"), Qt::CaseInsensitive) == 0
            || terminal_app.compare(QStringLiteral("iTerm2"), Qt::CaseInsensitive) == 0) {
            script = QStringLiteral(
                "tell application \"%1\"\n"
                "  activate\n"
                "  set newWin to (create window with default profile)\n"
                "  tell current session of newWin to write text \"%2\"\n"
                "end tell")
                .arg(terminal_app, as_escaped);
        } else {
            script = QStringLiteral("tell application \"%1\" to do script \"%2\"\n"
                                    "tell application \"%1\" to activate")
                         .arg(terminal_app, as_escaped);
        }
        // Run synchronously so we can surface osascript's exit code + stderr.
        // startDetached only fails if the process couldn't spawn at all; it
        // returns true even when AppleScript itself errors out, which made the
        // failure mode silent. Sync wait on osascript is fast (sub-second).
        QProcess proc;
        proc.start(QStringLiteral("/usr/bin/osascript"),
                   QStringList{QStringLiteral("-e"), script});
        if (!proc.waitForStarted(2000)) {
            QMessageBox::warning(parent, QStringLiteral("Goto Source"),
                                 QStringLiteral("Could not start osascript."));
            return;
        }
        proc.waitForFinished(5000);
        if (proc.exitStatus() != QProcess::NormalExit || proc.exitCode() != 0) {
            QString err = QString::fromLocal8Bit(proc.readAllStandardError()).trimmed();
            if (err.isEmpty()) {
                err = QStringLiteral("exit code %1").arg(proc.exitCode());
            }
            QMessageBox::warning(parent, QStringLiteral("Goto Source"),
                                 QStringLiteral("Failed to launch %1 in %2:\n%3")
                                     .arg(cmd, terminal_app, err));
        }
        return;
#else
        QMessageBox::warning(parent, QStringLiteral("Goto Source"),
            QStringLiteral("Terminal-mode launch is currently macOS-only. "
                           "Either uncheck \"Run editor in terminal\" or set "
                           "editor_command to a self-contained terminal "
                           "wrapper (e.g. \"xterm -e nvim\")."));
        return;
#endif
    }

    if (!QProcess::startDetached(program, parts)) {
        QMessageBox::warning(parent, QStringLiteral("Goto Source"), QStringLiteral("Failed to launch: %1").arg(cmd));
    }
}

bool prompt_new_project(QWidget *parent, QString &out_name, int &out_lang) {
    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("New Project"));

    auto *name_edit = new QLineEdit(&dlg);
    name_edit->setPlaceholderText(QStringLiteral("top_level"));

    auto *lang_combo = new QComboBox(&dlg);
    lang_combo->addItem(QStringLiteral("VHDL"), 0);
    lang_combo->addItem(QStringLiteral("SystemVerilog"), 1);

    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("&Name:"), name_edit);
    form->addRow(QStringLiteral("&Language:"), lang_combo);

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);

    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(buttons);

    if (dlg.exec() != QDialog::Accepted) {
        return false;
    }
    out_name = name_edit->text().trimmed();
    if (out_name.isEmpty()) {
        out_name = QStringLiteral("top_level");
    }
    out_lang = lang_combo->currentData().toInt();
    return true;
}

void prompt_preferences(QWidget *parent) {
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));

    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("Preferences"));

    auto *editor_edit = new QLineEdit(&dlg);
    editor_edit->setText(settings.value(QStringLiteral("editor_command"), QString()).toString());
    editor_edit->setPlaceholderText(QStringLiteral("e.g. nvim, code, zed"));

    auto *in_term_check = new QCheckBox(
        QStringLiteral("Run editor in terminal (required for nvim/vim)"), &dlg);
    in_term_check->setChecked(settings.value(QStringLiteral("editor_in_terminal"), false).toBool());

    auto *terminal_edit = new QLineEdit(&dlg);
    terminal_edit->setText(
        settings.value(QStringLiteral("terminal_app"), QStringLiteral("Terminal")).toString());
    terminal_edit->setPlaceholderText(QStringLiteral("Terminal, iTerm, Alacritty, kitty, Ghostty"));
    terminal_edit->setToolTip(QStringLiteral(
        "macOS only. Application name used by AppleScript `do script` to host "
        "the editor process. Must support `do script`."));

    auto *default_dir_edit = new QLineEdit(&dlg);
    default_dir_edit->setText(settings.value(QStringLiteral("default_open_dir"), QString()).toString());
    default_dir_edit->setPlaceholderText(QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation));
    auto *browse_btn = new QPushButton(QStringLiteral("Browse..."), &dlg);
    QObject::connect(browse_btn, &QPushButton::clicked, &dlg, [&dlg, default_dir_edit]() {
        QString seed = default_dir_edit->text();
        if (seed.isEmpty()) {
            seed = QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation);
        }
        QString picked = QFileDialog::getExistingDirectory(&dlg, QStringLiteral("Default Open Directory"), seed);
        if (!picked.isEmpty()) {
            default_dir_edit->setText(picked);
        }
    });
    auto *dir_row = new QHBoxLayout;
    dir_row->addWidget(default_dir_edit);
    dir_row->addWidget(browse_btn);

    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("&External editor command:"), editor_edit);
    form->addRow(QString(), in_term_check);
    form->addRow(QStringLiteral("&Terminal app (macOS):"), terminal_edit);
    form->addRow(QStringLiteral("&Default open directory:"), dir_row);

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);

    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(buttons);

    if (dlg.exec() == QDialog::Accepted) {
        settings.setValue(QStringLiteral("editor_command"), editor_edit->text());
        settings.setValue(QStringLiteral("editor_in_terminal"), in_term_check->isChecked());
        settings.setValue(QStringLiteral("terminal_app"), terminal_edit->text().trimmed());
        settings.setValue(QStringLiteral("default_open_dir"), default_dir_edit->text().trimmed());
    }
}

} // namespace

namespace {

using namespace hdlc;

// Tab on the completer popup accepts the highlighted candidate. Default
// QCompleter binds Return only.
class TabAcceptFilter : public QObject {
  public:
    TabAcceptFilter(QCompleter *c, QObject *parent) : QObject(parent), m_completer(c) {}

  protected:
    bool eventFilter(QObject *obj, QEvent *ev) override {
        if (ev->type() == QEvent::KeyPress) {
            auto *ke = static_cast<QKeyEvent *>(ev);
            if (ke->key() == Qt::Key_Tab && m_completer->popup()->isVisible()) {
                QModelIndex idx = m_completer->popup()->currentIndex();
                if (!idx.isValid()) {
                    idx = m_completer->completionModel()->index(0, 0);
                }
                if (idx.isValid()) {
                    QString text = idx.data(Qt::EditRole).toString();
                    m_completer->popup()->hide();
                    emit m_completer->activated(text);
                }
                return true;
            }
        }
        return QObject::eventFilter(obj, ev);
    }

  private:
    QCompleter *m_completer;
};

// Focus-out → commit. QPlainTextEdit has no direct focusOut signal.
class FocusOutFilter : public QObject {
  public:
    FocusOutFilter(std::function<void()> on_focus_out, QObject *parent)
        : QObject(parent), m_cb(std::move(on_focus_out)) {}

  protected:
    bool eventFilter(QObject *obj, QEvent *ev) override {
        if (ev->type() == QEvent::FocusOut)
            m_cb();
        return QObject::eventFilter(obj, ev);
    }

  private:
    std::function<void()> m_cb;
};

// Underlines the RHS of port-map lines that fail validation. The set of
// bad line indices is recomputed (Rust-side) on every idle parse.
class MiniEditorHighlighter : public QSyntaxHighlighter {
  public:
    explicit MiniEditorHighlighter(QTextDocument *doc) : QSyntaxHighlighter(doc) {}

    void setErrorLines(const QSet<int> &lines) {
        if (lines == m_error_lines)
            return;
        m_error_lines = lines;
        rehighlight();
    }

  protected:
    void highlightBlock(const QString &text) override {
        int blk = currentBlock().blockNumber();
        if (!m_error_lines.contains(blk))
            return;
        QTextCharFormat fmt;
        fmt.setUnderlineColor(QColor(220, 60, 60));
        fmt.setUnderlineStyle(QTextCharFormat::WaveUnderline);
        int start = 0;
        int len = text.length();
        int arrow = text.indexOf(QStringLiteral("=>"));
        if (arrow >= 0) {
            // VHDL form: underline RHS after `=>`.
            start = arrow + 2;
            while (start < text.length() && text[start].isSpace())
                ++start;
            len = text.length() - start;
        } else {
            // SV form `.name(value)`: underline the value between parens.
            int open = text.indexOf(QChar('('));
            int close = text.lastIndexOf(QChar(')'));
            if (open >= 0 && close > open) {
                start = open + 1;
                len = close - start;
            }
        }
        if (len > 0)
            setFormat(start, len, fmt);
    }

  private:
    QSet<int> m_error_lines;
};

// Main application window: owns the AppState model, the three-pane layout
// (sidebar / canvas / mini editor), menus, toolbar, and all signal wiring.
// Mini-editor state lives in members (was heap-allocated lambda captures
// when this was all one run_gui function).
class MainWindow : public QMainWindow {
  public:
    MainWindow() {
        {
            QScreen *screen = QGuiApplication::primaryScreen();
            const QRect avail = screen ? screen->availableGeometry() : QRect(0, 0, 1400, 900);
            const int w = std::max(1024, static_cast<int>(avail.width() * 0.85));
            const int h = std::max(720, static_cast<int>(avail.height() * 0.85));
            resize(w, h);
            move(avail.center() - QPoint(w / 2, h / 2));
        }
        m_state = new AppState(this);
        m_dirty_icon = make_dirty_icon();
        buildLayout();
        buildMenusAndToolbar();
        connectEditor();
        connectModelSignals();
        connectTreeSignals();
        statusBar()->showMessage(QStringLiteral("Ready"));
        update_window_title(this, m_state);
    }

  private:
    // --- UI construction ------------------------------------------------

    void buildLayout() {
        m_root_splitter = new QSplitter(Qt::Horizontal, this);

        // Sidebar: instance tree over module library.
        auto *sidebar_splitter = new QSplitter(Qt::Vertical, m_root_splitter);
        m_tree_model = new QStandardItemModel(this);
        m_tree_view = new QTreeView(sidebar_splitter);
        m_tree_view->setModel(m_tree_model);
        m_tree_view->setHeaderHidden(true);
        m_tree_view->setMinimumWidth(200);
        m_tree_view->setContextMenuPolicy(Qt::CustomContextMenu);
        m_tree_view->setSelectionMode(QAbstractItemView::SingleSelection);

        auto *library_label = new QLabel(QStringLiteral("Library"));
        library_label->setContentsMargins(4, 4, 4, 0);
        auto *library_view = new LibraryView(sidebar_splitter);
        m_library_model = new QStringListModel(this);
        library_view->setModel(m_library_model);
        auto *library_container = new QWidget(sidebar_splitter);
        auto *lib_layout = new QVBoxLayout(library_container);
        lib_layout->setContentsMargins(0, 0, 0, 0);
        lib_layout->setSpacing(0);
        lib_layout->addWidget(library_label);
        lib_layout->addWidget(library_view);

        sidebar_splitter->addWidget(m_tree_view);
        sidebar_splitter->addWidget(library_container);
        sidebar_splitter->setSizes({500, 300});

        // Canvas.
        auto *scene = new QGraphicsScene(m_root_splitter);
        scene->setSceneRect(-2000, -2000, 4000, 4000);
        auto *canvas = new CanvasView(scene, m_state, m_root_splitter);
        canvas->setMinimumWidth(600);
        m_canvas_layer = std::make_unique<CanvasLayer>(scene, m_state);
        canvas->setWireTool(m_canvas_layer->wireTool());
        canvas->setCanvasLayer(m_canvas_layer.get());
        m_canvas = canvas;

        // Mini editor panel: toggle row above the buffer. Toggle flips
        // between per-instance editing (default) and top-level entity
        // editing. Panel stays visible so the Top Level button is always
        // reachable even when nothing is selected.
        auto *editor_panel = new QWidget(m_root_splitter);
        editor_panel->setMinimumWidth(300);
        auto *editor_layout = new QVBoxLayout(editor_panel);
        editor_layout->setContentsMargins(0, 0, 0, 0);
        editor_layout->setSpacing(2);
        m_editor_top_level_btn = new QPushButton(QStringLiteral("Top Level"), editor_panel);
        m_editor_top_level_btn->setCheckable(true);
        m_editor_top_level_btn->setToolTip(
            QStringLiteral("Edit the top-level entity declaration: add/remove ports and generics."));
        editor_layout->addWidget(m_editor_top_level_btn);
        m_editor = new QPlainTextEdit(editor_panel);
        // Stretch factor 1 so the editor absorbs all extra vertical space
        // when visible. The trailing stretch (factor 0) takes over when the
        // editor is hidden — without it, QVBoxLayout would center the lone
        // button.
        editor_layout->addWidget(m_editor, 1);
        editor_layout->addStretch();
        m_editor->hide(); // shown only with a selection or top-level mode
        {
            QFont f(QStringLiteral("Menlo"));
            f.setStyleHint(QFont::Monospace);
            f.setFixedPitch(true);
            m_editor->setFont(f);
        }

        m_root_splitter->addWidget(sidebar_splitter);
        m_root_splitter->addWidget(canvas);
        m_root_splitter->addWidget(editor_panel);
        m_root_splitter->setSizes({250, 800, 350});
        // Editor panel can be dragged narrow but not collapsed to zero —
        // otherwise the splitter handle disappears. The Show Editor toolbar
        // action restores it forcibly.
        m_root_splitter->setCollapsible(2, false);
        setCentralWidget(m_root_splitter);
    }

    void buildMenusAndToolbar() {
        auto *fileMenu = menuBar()->addMenu(QStringLiteral("&File"));
        auto *newAct = fileMenu->addAction(QStringLiteral("&New..."));
        newAct->setShortcut(QKeySequence::New);
        auto *openAct = fileMenu->addAction(QStringLiteral("&Open..."));
        openAct->setShortcut(QKeySequence::Open);
        m_recent_menu = fileMenu->addMenu(QStringLiteral("Open &Recent"));
        connect(m_recent_menu, &QMenu::aboutToShow, this, &MainWindow::rebuildRecentMenu);
        auto *addSourceAct = fileMenu->addAction(QStringLiteral("&Add HDL Source..."));
        addSourceAct->setShortcut(QKeySequence(Qt::CTRL | Qt::SHIFT | Qt::Key_O));
        auto *reloadAct = fileMenu->addAction(QStringLiteral("&Refresh Library"));
        reloadAct->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_R));
        auto *copySourcesAct = fileMenu->addAction(QStringLiteral("&Copy Sources to Project Dir"));
        fileMenu->addSeparator();
        auto *saveAct = fileMenu->addAction(QStringLiteral("&Save"));
        saveAct->setShortcut(QKeySequence::Save);
        auto *saveAsAct = fileMenu->addAction(QStringLiteral("Save &As..."));
        saveAsAct->setShortcut(QKeySequence::SaveAs);
        fileMenu->addSeparator();
        auto *generateAct = fileMenu->addAction(QStringLiteral("&Generate HDL..."));
        generateAct->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_G));
        fileMenu->addSeparator();
        auto *prefsAct = fileMenu->addAction(QStringLiteral("&Preferences..."));
        fileMenu->addSeparator();
        auto *exitAct = fileMenu->addAction(QStringLiteral("E&xit"));
        exitAct->setShortcut(QKeySequence::Quit);

        // Edit menu — operations on the selected instance.
        auto *editMenu = menuBar()->addMenu(QStringLiteral("&Edit"));
        m_undo_act = editMenu->addAction(QStringLiteral("&Undo"));
        m_undo_act->setShortcut(QKeySequence::Undo);
        m_redo_act = editMenu->addAction(QStringLiteral("&Redo"));
        m_redo_act->setShortcut(QKeySequence::Redo);
        editMenu->addSeparator();
        auto *renameInstAct = editMenu->addAction(QStringLiteral("Re&name Instance..."));
        auto *deleteInstAct = editMenu->addAction(QStringLiteral("&Delete Instance"));
        editMenu->addSeparator();
        auto *matchByNameAct = editMenu->addAction(QStringLiteral("&Match Ports by Name"));
        matchByNameAct->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_M));
        matchByNameAct->setToolTip(
            QStringLiteral("Connect unmapped ports on the selected instance to matching top-level "
                           "ports (same name + direction + type)"));

        // View menu — canvas navigation + diagnostics.
        auto *viewMenu = menuBar()->addMenu(QStringLiteral("&View"));
        auto *zoomFitAct = viewMenu->addAction(QStringLiteral("Zoom to &Fit"));
        zoomFitAct->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_0));
        auto *issuesAct = viewMenu->addAction(QStringLiteral("Validation &Issues..."));

        // Help menu — the canvas is gesture-heavy; give the gestures a home.
        auto *helpMenu = menuBar()->addMenu(QStringLiteral("&Help"));
        auto *controlsAct = helpMenu->addAction(QStringLiteral("&Canvas Controls"));

        // Toolbar shares QAction pointers with the File menu so shortcuts,
        // enable-state, and icons stay in sync.
        auto *fileToolbar = addToolBar(QStringLiteral("File"));
        fileToolbar->setObjectName(QStringLiteral("FileToolbar"));
        fileToolbar->setMovable(false);
        fileToolbar->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
        newAct->setIconText(QStringLiteral("New"));
        openAct->setIconText(QStringLiteral("Open"));
        addSourceAct->setIconText(QStringLiteral("Add HDL"));
        reloadAct->setIconText(QStringLiteral("Refresh"));
        saveAct->setIconText(QStringLiteral("Save"));
        fileToolbar->addAction(newAct);
        fileToolbar->addAction(openAct);
        fileToolbar->addAction(addSourceAct);
        fileToolbar->addAction(reloadAct);
        fileToolbar->addSeparator();
        fileToolbar->addAction(saveAct);

        // Force-reopen the editor panel if the user dragged it narrow.
        auto *showEditorAct = new QAction(QStringLiteral("Show Editor"), this);
        showEditorAct->setToolTip(QStringLiteral("Restore the editor panel to its default width."));
        showEditorAct->setShortcut(QKeySequence(QStringLiteral("Ctrl+\\")));
        fileToolbar->addSeparator();
        fileToolbar->addAction(showEditorAct);

        connect(newAct, &QAction::triggered, this, &MainWindow::onNewProject);
        connect(openAct, &QAction::triggered, this, &MainWindow::onOpenProject);
        connect(addSourceAct, &QAction::triggered, this, &MainWindow::onAddSource);
        connect(saveAct, &QAction::triggered, this, &MainWindow::onSaveProject);
        connect(saveAsAct, &QAction::triggered, this, [this]() { saveAsProject(); });
        connect(generateAct, &QAction::triggered, this, &MainWindow::onGenerate);
        connect(reloadAct, &QAction::triggered, this, &MainWindow::onReloadLibrary);
        connect(copySourcesAct, &QAction::triggered, this, &MainWindow::onCopySources);
        connect(prefsAct, &QAction::triggered, this, [this]() { prompt_preferences(this); });
        // close() (not qApp->quit()) so closeEvent can catch unsaved changes.
        connect(exitAct, &QAction::triggered, this, &MainWindow::close);
        connect(matchByNameAct, &QAction::triggered, this, &MainWindow::onMatchByName);
        connect(renameInstAct, &QAction::triggered, this, &MainWindow::onRenameSelected);
        connect(deleteInstAct, &QAction::triggered, this, &MainWindow::onDeleteSelected);
        connect(showEditorAct, &QAction::triggered, this, &MainWindow::showEditorPanel);
        connect(zoomFitAct, &QAction::triggered, this, [this]() {
            if (m_canvas)
                m_canvas->zoomToFit();
        });
        connect(issuesAct, &QAction::triggered, this, &MainWindow::onShowValidationIssues);
        connect(controlsAct, &QAction::triggered, this, &MainWindow::onShowCanvasControls);
        connect(m_undo_act, &QAction::triggered, this, [this]() {
            if (m_state->undo()) {
                statusBar()->showMessage(QStringLiteral("Undo"), 1500);
            }
        });
        connect(m_redo_act, &QAction::triggered, this, [this]() {
            if (m_state->redo()) {
                statusBar()->showMessage(QStringLiteral("Redo"), 1500);
            }
        });
    }

    void connectEditor() {
        // Inline syntax highlighter — red squiggles on bad RHS lines,
        // recomputed on a 300 ms idle debounce.
        m_highlighter = new MiniEditorHighlighter(m_editor->document());

        // RHS / dot-completer. Popup driven manually since QPlainTextEdit
        // doesn't auto-attach to QCompleter the way QLineEdit does.
        m_completer_model = new QStringListModel(m_editor);
        m_completer = new QCompleter(m_completer_model, m_editor);
        m_completer->setWidget(m_editor);
        m_completer->setCompletionMode(QCompleter::PopupCompletion);
        m_completer->setCaseSensitivity(Qt::CaseInsensitive);
        m_completer->popup()->installEventFilter(new TabAcceptFilter(m_completer, m_editor));

        m_parse_timer = new QTimer(m_editor);
        m_parse_timer->setSingleShot(true);
        m_parse_timer->setInterval(300);

        // Toggle: enter top-level mode → deselect any instance and load the
        // top-level entity buffer. Exit → repopulate from the still-selected
        // instance (or hide the editor if none).
        connect(m_editor_top_level_btn, &QPushButton::toggled, this, [this](bool checked) {
            commitEditor(); // flush in-flight edit before swapping
            m_top_level_mode = checked;
            if (checked) {
                m_state->set_selected_instance(QString());
                m_editor_inst.clear();
            }
            repopulateEditor();
        });

        // textChanged just restarts the debounce timer; all real work waits
        // for idle.
        connect(m_editor, &QPlainTextEdit::textChanged, this, [this]() {
            if (m_editor_suppressing)
                return;
            m_editor_editing = true;
            m_parse_timer->start();
        });

        connect(m_parse_timer, &QTimer::timeout, this, &MainWindow::onParseTimeout);

        connect(m_completer, QOverload<const QString &>::of(&QCompleter::activated), this,
                [this](const QString &text) {
                    QTextCursor c = m_editor->textCursor();
                    int n = m_completer->completionPrefix().length();
                    if (n > 0) {
                        c.movePosition(QTextCursor::Left, QTextCursor::KeepAnchor, n);
                    }
                    c.insertText(text);
                    m_editor->setTextCursor(c);
                });

        // Focus-out → commit.
        m_editor->installEventFilter(new FocusOutFilter([this]() { commitEditor(); }, m_editor));

        // Ctrl+Return also commits.
        auto *commit_sc = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Return), m_editor);
        commit_sc->setContext(Qt::WidgetWithChildrenShortcut);
        connect(commit_sc, &QShortcut::activated, this, &MainWindow::commitEditor);
    }

    void connectModelSignals() {
        // Title + validation reactive.
        connect(m_state, &AppState::project_nameChanged, this,
                [this]() { update_window_title(this, m_state); });
        connect(m_state, &AppState::dirtyChanged, this, [this]() { update_window_title(this, m_state); });
        connect(m_state, &AppState::validation_changed, this, [this]() {
            int errs = m_state->validation_error_count();
            int warns = m_state->validation_warning_count();
            statusBar()->showMessage(QStringLiteral("%1 error(s), %2 warning(s)").arg(errs).arg(warns));
        });

        // Undo/redo enable-state tracks every mutation signal.
        refreshUndoActions();
        connect(m_state, &AppState::project_loaded, this, &MainWindow::refreshUndoActions);
        connect(m_state, &AppState::port_map_changed, this,
                [this](const QString &, const QString &) { refreshUndoActions(); });
        connect(m_state, &AppState::port_map_changed_bulk, this, &MainWindow::refreshUndoActions);
        connect(m_state, &AppState::instance_added, this, [this](const QString &) { refreshUndoActions(); });
        connect(m_state, &AppState::instance_removed, this, [this](const QString &) { refreshUndoActions(); });

        // Sidebar + canvas reactive.
        connect(m_state, &AppState::project_loaded, this, [this]() {
            refreshSidebar();
            m_canvas_layer->rebuild();
        });
        connect(m_state, &AppState::instance_added, this, [this](const QString &name) {
            refreshSidebar();
            m_canvas_layer->onInstanceAdded(name);
        });
        connect(m_state, &AppState::instance_removed, this, [this](const QString &name) {
            refreshSidebar();
            m_canvas_layer->onInstanceRemoved(name);
        });
        connect(m_state, &AppState::instance_moved, this,
                [this](const QString &name, double x, double y) { m_canvas_layer->onInstanceMoved(name, x, y); });
        connect(m_state, &AppState::port_map_changed, this,
                [this](const QString &inst, const QString &) { m_canvas_layer->onPortMapChanged(inst); });
        connect(m_state, &AppState::port_map_changed_bulk, this,
                [this]() { m_canvas_layer->onPortMapChangedBulk(); });
        // Aliases only rename/recolor nets — wires, not pin layout.
        connect(m_state, &AppState::alias_changed, this,
                [this](const QString &) { m_canvas_layer->rebuildWires(); });
        connect(m_state, &AppState::library_changed, this, &MainWindow::refreshSidebar);

        // Module re-parse: watch every library path and auto-reload when the
        // underlying file changes. The reload drops stale port_map entries
        // and flags affected instances dirty; the user reviews and either
        // reconnects or clears the dirty flag.
        m_fs_watcher = new QFileSystemWatcher(this);
        connect(m_state, &AppState::library_changed, this, &MainWindow::refreshWatcher);
        connect(m_state, &AppState::project_loaded, this, &MainWindow::refreshWatcher);
        connect(m_fs_watcher, &QFileSystemWatcher::fileChanged, this, [this](const QString &path) {
            // Some editors rename-swap to save — re-add the path after a
            // brief delay in case the watcher lost it.
            QTimer::singleShot(50, this, [this]() {
                m_state->reload_library();
                refreshWatcher();
            });
            statusBar()->showMessage(
                QStringLiteral("Source changed: %1 — reloading").arg(QFileInfo(path).fileName()), 3000);
        });

        connect(m_state, &AppState::selection_changed, this, &MainWindow::onSelectionChanged);

        // Mini-editor refresh on model changes — only when not being
        // actively edited; once the user is typing we wait for focus-out.
        auto editor_model_changed = [this]() {
            if (m_editor_editing)
                return;
            repopulateEditor();
        };
        connect(m_state, &AppState::port_map_changed, this,
                [editor_model_changed](const QString &, const QString &) { editor_model_changed(); });
        connect(m_state, &AppState::port_map_changed_bulk, this, editor_model_changed);
        connect(m_state, &AppState::project_loaded, this, [this]() {
            m_editor_inst.clear();
            repopulateEditor();
        });
    }

    void connectTreeSignals() {
        // Single-click → set selection via AppState.
        connect(m_tree_view, &QTreeView::clicked, this, [this](const QModelIndex &index) {
            QString name = index.data(Qt::UserRole).toString();
            if (name.isEmpty()) {
                return;
            }
            m_state->set_selected_instance(name);
        });

        // Double-click → goto source.
        connect(m_tree_view, &QTreeView::doubleClicked, this, [this](const QModelIndex &index) {
            QString inst_name = index.data(Qt::UserRole).toString();
            if (inst_name.isEmpty()) {
                return;
            }
            int idx = find_instance_index(m_state, inst_name);
            if (idx < 0) {
                return;
            }
            QString src = m_state->instance_source_path(idx);
            launch_goto_source(this, src);
        });

        connect(m_tree_view, &QTreeView::customContextMenuRequested, this, &MainWindow::onTreeContextMenu);
    }

    // --- Mini editor ------------------------------------------------------

    void repopulateEditor() {
        m_editor_suppressing = true;
        if (m_top_level_mode) {
            m_editor->setPlainText(m_state->top_level_buffer());
            m_editor->show();
        } else if (m_editor_inst.isEmpty()) {
            m_editor->clear();
            m_editor->hide();
        } else {
            m_editor->setPlainText(m_state->instance_buffer(m_editor_inst));
            m_editor->show();
        }
        m_editor_suppressing = false;
        m_editor_editing = false;
        m_highlighter->setErrorLines({});
        m_completer->popup()->hide();
        m_parse_timer->stop();
    }

    void commitEditor() {
        if (!m_editor_editing)
            return; // nothing to commit
        if (m_top_level_mode) {
            if (!m_state->commit_top_level_buffer(m_editor->toPlainText())) {
                QString err = m_state->last_error();
                statusBar()->showMessage(
                    QStringLiteral("Top-level: %1").arg(err.isEmpty() ? QStringLiteral("commit refused") : err),
                    5000);
                return;
            }
            m_editor_editing = false;
            statusBar()->showMessage(QStringLiteral("Top-level entity updated"), 2000);
            return;
        }
        if (m_editor_inst.isEmpty())
            return;
        FfiEditorIssues issues = m_state->commit_instance_buffer(m_editor_inst, m_editor->toPlainText());
        if (!issues.messages.empty()) {
            // Refuse silently: squiggles + status bar already told the user.
            // Editor stays as-is; user fixes and retries.
            statusBar()->showMessage(QStringLiteral("Mini editor: %1 parse error(s) — fix to commit")
                                         .arg(static_cast<int>(issues.messages.size())),
                                     4000);
            return;
        }
        // Don't re-render: that would jump the cursor and clobber the user's
        // formatting. Just mark the buffer clean. Column normalization will
        // happen the next time selection_changed switches away.
        m_editor_editing = false;
        statusBar()->showMessage(QStringLiteral("Mini editor changes applied"), 2000);
    }

    void onParseTimeout() {
        if (m_top_level_mode) {
            // Top-level grammar isn't checked live; commit-time errors land
            // in the status bar instead of inline squiggles.
            m_highlighter->setErrorLines({});
            m_completer->popup()->hide();
            return;
        }
        FfiEditorIssues issues = m_state->check_instance_buffer(m_editor->toPlainText());
        QSet<int> err_lines;
        for (int line : issues.lines)
            err_lines.insert(line);
        m_highlighter->setErrorLines(err_lines);
        if (issues.lines.empty()) {
            statusBar()->clearMessage();
        } else {
            statusBar()->showMessage(QStringLiteral("Mini editor: %1 parse error(s)")
                                         .arg(static_cast<int>(issues.lines.size())));
        }

        // Completer popup based on cursor context.
        QTextCursor cur = m_editor->textCursor();
        QString line = cur.block().text();
        int pos_in_block = cur.positionInBlock();
        QString before = line.left(pos_in_block);
        const std::string before_utf8 = before.toStdString();
        FfiCompletionContext ctx = completion_context_ffi(before_utf8);

        if (ctx.kind == 0) {
            m_completer->popup()->hide();
            return;
        }

        auto to_qstring = [](const rust::String &r) {
            return QString::fromUtf8(r.data(), static_cast<int>(r.size()));
        };
        rust::Vec<rust::String> cands =
            (ctx.kind == 2) ? m_state->editor_port_candidates(to_qstring(ctx.instance))
                            : m_state->editor_rhs_candidates(m_editor_inst);
        QStringList items;
        items.reserve(static_cast<int>(cands.size()));
        for (const rust::String &c : cands)
            items << to_qstring(c);
        m_completer_model->setStringList(items);
        m_completer->setCompletionPrefix(to_qstring(ctx.prefix));
        if (m_completer->completionCount() == 0) {
            m_completer->popup()->hide();
            return;
        }
        m_completer->popup()->setCurrentIndex(m_completer->completionModel()->index(0, 0));
        QRect rect = m_editor->cursorRect();
        rect.setWidth(m_completer->popup()->sizeHintForColumn(0) +
                      m_completer->popup()->verticalScrollBar()->sizeHint().width());
        m_completer->complete(rect);
    }

    // --- Reactive helpers ---------------------------------------------------

    void refreshSidebar() {
        rebuild_tree_model(m_tree_model, m_state, m_dirty_icon);
        rebuild_library_model(m_library_model, m_state);
        m_tree_view->expandAll();
    }

    void refreshUndoActions() {
        m_undo_act->setEnabled(m_state->can_undo());
        m_redo_act->setEnabled(m_state->can_redo());
    }

    void refreshWatcher() {
        if (!m_fs_watcher->files().isEmpty()) {
            m_fs_watcher->removePaths(m_fs_watcher->files());
        }
        int n = m_state->library_path_count();
        QStringList existing;
        for (int i = 0; i < n; ++i) {
            QString p = m_state->library_path(i);
            if (QFileInfo::exists(p))
                existing << p;
        }
        if (!existing.isEmpty())
            m_fs_watcher->addPaths(existing);
    }

    void onSelectionChanged(const QString &name) {
        // Commit any outgoing edit against the previous instance before
        // switching so the user doesn't lose work.
        commitEditor();
        // Selecting an instance kicks the editor out of top-level mode.
        if (!name.isEmpty() && m_top_level_mode) {
            m_top_level_mode = false;
            QSignalBlocker b(m_editor_top_level_btn);
            m_editor_top_level_btn->setChecked(false);
        }
        m_canvas_layer->refreshSelectionHighlight();
        m_editor_inst = name;
        repopulateEditor();
        // Sync sidebar tree row.
        for (int row = 0; row < m_tree_model->rowCount(); ++row) {
            auto *root_item = m_tree_model->item(row);
            for (int c = 0; c < root_item->rowCount(); ++c) {
                auto *child = root_item->child(c);
                if (child->data(Qt::UserRole).toString() == name) {
                    m_tree_view->setCurrentIndex(child->index());
                    return;
                }
            }
        }
    }

    void onTreeContextMenu(const QPoint &pos) {
        QModelIndex idx = m_tree_view->indexAt(pos);
        if (!idx.isValid()) {
            return;
        }
        QString inst_name = idx.data(Qt::UserRole).toString();
        if (inst_name.isEmpty()) {
            return;
        }
        QMenu menu(m_tree_view);
        QAction *renameAct = menu.addAction(QStringLiteral("Rename..."));
        QAction *deleteAct = menu.addAction(QStringLiteral("Delete"));
        QAction *chosen = menu.exec(m_tree_view->viewport()->mapToGlobal(pos));
        if (chosen == renameAct) {
            promptRenameInstance(inst_name);
        } else if (chosen == deleteAct) {
            promptDeleteInstance(inst_name);
        }
    }

    void promptRenameInstance(const QString &inst_name) {
        bool ok = false;
        QString new_name = QInputDialog::getText(this, QStringLiteral("Rename Instance"),
                                                 QStringLiteral("New name:"), QLineEdit::Normal, inst_name, &ok);
        if (!ok || new_name.trimmed().isEmpty() || new_name == inst_name) {
            return;
        }
        if (!m_state->rename_instance(inst_name, new_name.trimmed())) {
            show_state_error(this, m_state, QStringLiteral("Rename"));
        }
    }

    void promptDeleteInstance(const QString &inst_name) {
        auto btn = QMessageBox::question(this, QStringLiteral("Delete Instance"),
                                         QStringLiteral("Delete instance %1?").arg(inst_name));
        if (btn != QMessageBox::Yes) {
            return;
        }
        if (!m_state->remove_instance(inst_name)) {
            show_state_error(this, m_state, QStringLiteral("Delete"));
        }
    }

    // Edit-menu variants act on the current selection.
    void onRenameSelected() {
        QString sel = m_state->selected_instance();
        if (sel.isEmpty()) {
            statusBar()->showMessage(QStringLiteral("Rename: select an instance first"), 3000);
            return;
        }
        promptRenameInstance(sel);
    }

    void onDeleteSelected() {
        QString sel = m_state->selected_instance();
        if (sel.isEmpty()) {
            statusBar()->showMessage(QStringLiteral("Delete: select an instance first"), 3000);
            return;
        }
        promptDeleteInstance(sel);
    }

    // --- File / project actions ---------------------------------------------

    /// Offer to save unsaved changes before a project-discarding action.
    /// Returns true to proceed, false if the user cancelled (or a chosen
    /// save failed).
    bool maybeSaveDirty() {
        if (!m_state->has_project() || !m_state->getDirty()) {
            return true;
        }
        auto btn = QMessageBox::warning(
            this, QStringLiteral("Unsaved Changes"),
            QStringLiteral("Project \"%1\" has unsaved changes.").arg(m_state->getProject_name()),
            QMessageBox::Save | QMessageBox::Discard | QMessageBox::Cancel, QMessageBox::Save);
        if (btn == QMessageBox::Cancel) {
            return false;
        }
        if (btn == QMessageBox::Discard) {
            return true;
        }
        // Save: fall back to Save As when there's no path yet (or save failed).
        return m_state->save_project() || saveAsProject();
    }

    void onNewProject() {
        if (!maybeSaveDirty()) {
            return;
        }
        QString name;
        int lang = 0;
        if (!prompt_new_project(this, name, lang)) {
            return;
        }
        if (!m_state->new_project(name, lang)) {
            show_state_error(this, m_state, QStringLiteral("New Project"));
            return;
        }
        statusBar()->showMessage(QStringLiteral("Created new project: %1").arg(name), 3000);
    }

    void openProjectPath(const QString &path) {
        if (!m_state->open_project(path)) {
            show_state_error(this, m_state, QStringLiteral("Open Project"));
            return;
        }
        add_recent_project(path);
        statusBar()->showMessage(QStringLiteral("Opened %1").arg(path), 3000);
    }

    void onOpenProject() {
        if (!maybeSaveDirty()) {
            return;
        }
        QString path = QFileDialog::getOpenFileName(this, QStringLiteral("Open Project"), default_open_dir(),
                                                    QStringLiteral("HDL Compose Projects (*.hdlc)"));
        if (path.isEmpty()) {
            return;
        }
        openProjectPath(path);
    }

    void rebuildRecentMenu() {
        m_recent_menu->clear();
        const QStringList recents = recent_projects();
        for (const QString &p : recents) {
            QAction *act = m_recent_menu->addAction(p);
            connect(act, &QAction::triggered, this, [this, p]() {
                if (maybeSaveDirty()) {
                    openProjectPath(p);
                }
            });
        }
        if (recents.isEmpty()) {
            m_recent_menu->addAction(QStringLiteral("(empty)"))->setEnabled(false);
            return;
        }
        m_recent_menu->addSeparator();
        QAction *clearAct = m_recent_menu->addAction(QStringLiteral("Clear Menu"));
        connect(clearAct, &QAction::triggered, this, []() {
            QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
            settings.remove(QStringLiteral("recent_projects"));
        });
    }

    void onAddSource() {
        if (!m_state->has_project()) {
            QMessageBox::information(this, QStringLiteral("Add HDL Source"),
                                     QStringLiteral("Create or open a project first."));
            return;
        }
        QStringList paths =
            QFileDialog::getOpenFileNames(this, QStringLiteral("Add HDL Source(s)"), default_open_dir(),
                                          QStringLiteral("HDL sources (*.vhd *.vhdl *.v *.sv);;All files (*)"));
        if (paths.isEmpty()) {
            return;
        }
        int added = 0;
        QStringList failed;
        for (const QString &p : paths) {
            if (m_state->add_library_path(p)) {
                added++;
            } else {
                QString err = m_state->last_error();
                failed << (err.isEmpty() ? p : QStringLiteral("%1 (%2)").arg(p, err));
            }
        }
        if (!failed.isEmpty()) {
            QMessageBox::warning(this, QStringLiteral("Add HDL Source"),
                                 QStringLiteral("Failed to add:\n%1").arg(failed.join(QChar('\n'))));
        }
        statusBar()->showMessage(QStringLiteral("Added %1 source(s)").arg(added), 3000);
    }

    bool saveAsProject() {
        QString path = QFileDialog::getSaveFileName(this, QStringLiteral("Save Project As"), default_open_dir(),
                                                    QStringLiteral("HDL Compose Projects (*.hdlc)"));
        if (path.isEmpty()) {
            return false;
        }
        if (!path.endsWith(QStringLiteral(".hdlc"), Qt::CaseInsensitive)) {
            path += QStringLiteral(".hdlc");
        }
        if (!m_state->save_project_as(path)) {
            show_state_error(this, m_state, QStringLiteral("Save Project"));
            return false;
        }
        add_recent_project(path);
        statusBar()->showMessage(QStringLiteral("Saved to %1").arg(path), 3000);
        return true;
    }

    void onSaveProject() {
        if (!m_state->has_project()) {
            QMessageBox::information(this, QStringLiteral("Save"), QStringLiteral("No project to save."));
            return;
        }
        if (m_state->save_project()) {
            statusBar()->showMessage(QStringLiteral("Saved"), 3000);
        } else {
            saveAsProject();
        }
    }

    void onGenerate() {
        if (!m_state->has_project()) {
            QMessageBox::information(this, QStringLiteral("Generate HDL"), QStringLiteral("No project loaded."));
            return;
        }
        int lang = m_state->project_language();
        QString filter;
        QString lang_label;
        switch (lang) {
        case 0:
            filter = QStringLiteral("VHDL (*.vhd *.vhdl)");
            lang_label = QStringLiteral("VHDL");
            break;
        case 1:
            filter = QStringLiteral("SystemVerilog (*.sv *.v)");
            lang_label = QStringLiteral("SystemVerilog");
            break;
        default:
            QMessageBox::warning(this, QStringLiteral("Generate HDL"), QStringLiteral("Unknown project language."));
            return;
        }
        QString suggested = m_state->suggest_codegen_path();
        QString path =
            QFileDialog::getSaveFileName(this, QStringLiteral("Generate %1").arg(lang_label), suggested, filter);
        if (path.isEmpty()) {
            return;
        }
        if (m_state->generate_code(path)) {
            statusBar()->showMessage(QStringLiteral("Generated %1").arg(path), 5000);
        } else {
            show_state_error(this, m_state, QStringLiteral("Generate HDL"));
        }
    }

    void onReloadLibrary() {
        if (!m_state->has_project()) {
            QMessageBox::information(this, QStringLiteral("Refresh Library"), QStringLiteral("No project loaded."));
            return;
        }
        if (m_state->reload_library()) {
            statusBar()->showMessage(QStringLiteral("Library refreshed"), 3000);
        } else {
            show_state_error(this, m_state, QStringLiteral("Refresh Library"));
        }
    }

    void onCopySources() {
        QString proj_path = m_state->current_project_path();
        if (proj_path.isEmpty()) {
            QMessageBox::information(this, QStringLiteral("Copy Sources"),
                                     QStringLiteral("Save the project first so we know where to copy to."));
            return;
        }
        QDir proj_dir = QFileInfo(proj_path).absoluteDir();
        int n = m_state->library_path_count();
        if (n == 0) {
            QMessageBox::information(this, QStringLiteral("Copy Sources"),
                                     QStringLiteral("No library sources to copy."));
            return;
        }
        auto btn = QMessageBox::question(
            this, QStringLiteral("Copy Sources"),
            QStringLiteral("Copy %1 source file(s) into %2?").arg(n).arg(proj_dir.absolutePath()));
        if (btn != QMessageBox::Yes) {
            return;
        }
        int copied = 0;
        QStringList failures;
        QStringList originals;
        for (int i = 0; i < n; ++i) {
            originals << m_state->library_path(i);
        }
        for (const QString &src : originals) {
            QFileInfo src_info(src);
            if (!src_info.exists()) {
                failures << QStringLiteral("%1 (missing)").arg(src);
                continue;
            }
            QString target = proj_dir.absoluteFilePath(src_info.fileName());
            if (QFileInfo(target) == src_info) {
                continue; // already in project dir
            }
            if (QFile::exists(target)) {
                auto overwrite = QMessageBox::question(this, QStringLiteral("Overwrite?"),
                                                       QStringLiteral("%1 exists. Overwrite?").arg(target));
                if (overwrite != QMessageBox::Yes) {
                    failures << QStringLiteral("%1 (skipped)").arg(src);
                    continue;
                }
                QFile::remove(target);
            }
            if (!QFile::copy(src, target)) {
                failures << QStringLiteral("%1 (copy failed)").arg(src);
                continue;
            }
            m_state->remove_library_path(src);
            m_state->add_library_path(target);
            copied++;
        }
        QString msg = QStringLiteral("Copied %1 of %2 file(s).").arg(copied).arg(n);
        if (!failures.isEmpty()) {
            msg += QStringLiteral("\n\nIssues:\n%1").arg(failures.join(QChar('\n')));
        }
        QMessageBox::information(this, QStringLiteral("Copy Sources"), msg);
    }

    void onMatchByName() {
        QString sel = m_state->selected_instance();
        if (sel.isEmpty()) {
            statusBar()->showMessage(QStringLiteral("Match by Name: select an instance first"), 3000);
            return;
        }
        int count = m_state->match_by_name(sel);
        if (count > 0) {
            statusBar()->showMessage(QStringLiteral("Matched %1 port(s) by name").arg(count), 3000);
        } else {
            statusBar()->showMessage(QStringLiteral("No matching top-level ports found for '%1'").arg(sel), 3000);
        }
    }

    // Restore the editor panel to its default share of the window.
    void showEditorPanel() {
        QList<int> sizes = m_root_splitter->sizes();
        int total = 0;
        for (int s : sizes)
            total += s;
        if (total <= 0)
            return;
        // Give the editor panel ~25% of the splitter, leaving the rest split
        // 25/75 between sidebar and canvas.
        int editor_w = static_cast<int>(total * 0.25);
        if (editor_w < 300)
            editor_w = (std::min)(300, total - 100);
        int remaining = total - editor_w;
        int sidebar_w = static_cast<int>(remaining * 0.25);
        int canvas_w = remaining - sidebar_w;
        m_root_splitter->setSizes({sidebar_w, canvas_w, editor_w});
    }

    // Full diagnostic text behind the status bar's error/warning counts.
    void onShowValidationIssues() {
        rust::Vec<rust::String> msgs = m_state->validation_messages();
        QString text;
        for (const rust::String &m : msgs) {
            text += QString::fromUtf8(m.data(), static_cast<int>(m.size()));
            text += QChar('\n');
        }
        if (text.isEmpty()) {
            text = QStringLiteral("No validation issues.");
        }
        QDialog dlg(this);
        dlg.setWindowTitle(QStringLiteral("Validation Issues"));
        auto *view = new QPlainTextEdit(&dlg);
        view->setReadOnly(true);
        view->setPlainText(text);
        auto *buttons = new QDialogButtonBox(QDialogButtonBox::Close, &dlg);
        QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);
        auto *layout = new QVBoxLayout(&dlg);
        layout->addWidget(view);
        layout->addWidget(buttons);
        dlg.resize(640, 360);
        dlg.exec();
    }

    void onShowCanvasControls() {
        QMessageBox box(this);
        box.setWindowTitle(QStringLiteral("Canvas Controls"));
        box.setTextFormat(Qt::RichText);
        box.setText(QStringLiteral(
            "<b>Wiring</b><br>"
            "Click a pin, then click another pin — or drag pin to pin. Esc cancels.<br>"
            "Input-to-input wiring joins loads on one net (tool stays armed for fan-out).<br>"
            "Hover a wire to highlight its whole net.<br><br>"
            "<b>Right-click</b><br>"
            "Wire &rarr; set net alias (signal name in generated HDL).<br>"
            "Pin &rarr; promote to top-level port, connect slice, clear connection.<br>"
            "Module body &rarr; group ports into an interface bundle.<br>"
            "Bundle row: click folds/unfolds, right-click ungroups.<br><br>"
            "<b>Selection &amp; delete</b><br>"
            "Click selects; empty-canvas click deselects. Shift+click a top port to multi-select.<br>"
            "Delete / Backspace removes selected wires, instances, and top ports.<br><br>"
            "<b>Navigation</b><br>"
            "Middle-drag pans. Ctrl+scroll zooms. F or Ctrl+0 zooms to fit.<br>"
            "Drag a top port vertically to reposition it along the edge.<br><br>"
            "<b>Mini editor</b> (right pane)<br>"
            "Ctrl+Return or focus-out commits. Tab accepts a completion.<br>"
            "\"Top Level\" button edits the top entity's ports and generics.<br><br>"
            "<b>Sidebar</b><br>"
            "Drag a library module onto the canvas to place it.<br>"
            "Double-click an instance to open its source (File &rarr; Preferences sets the editor)."));
        box.exec();
    }

  protected:
    void closeEvent(QCloseEvent *event) override {
        if (maybeSaveDirty()) {
            event->accept();
        } else {
            event->ignore();
        }
    }

  private:
    // --- Members --------------------------------------------------------

    AppState *m_state = nullptr;
    QIcon m_dirty_icon;
    QSplitter *m_root_splitter = nullptr;
    QStandardItemModel *m_tree_model = nullptr;
    QTreeView *m_tree_view = nullptr;
    QStringListModel *m_library_model = nullptr;
    CanvasView *m_canvas = nullptr;
    QMenu *m_recent_menu = nullptr;
    std::unique_ptr<CanvasLayer> m_canvas_layer;
    QPushButton *m_editor_top_level_btn = nullptr;
    QPlainTextEdit *m_editor = nullptr;
    MiniEditorHighlighter *m_highlighter = nullptr;
    QStringListModel *m_completer_model = nullptr;
    QCompleter *m_completer = nullptr;
    QTimer *m_parse_timer = nullptr;
    QFileSystemWatcher *m_fs_watcher = nullptr;
    QAction *m_undo_act = nullptr;
    QAction *m_redo_act = nullptr;

    // Mini-editor state.
    bool m_top_level_mode = false;
    QString m_editor_inst; // instance shown in the buffer; empty = none
    bool m_editor_editing = false;     // user typing; suppress auto-repopulate
    bool m_editor_suppressing = false; // programmatic setPlainText guard
};

} // anonymous namespace

extern "C" int run_gui(int *argc, char **argv) {
    // HiDPI: pass-through fractional scale factors (e.g. 2x Retina) without
    // rounding — keeps fonts and pixmaps crisp on macOS.
    QGuiApplication::setHighDpiScaleFactorRoundingPolicy(Qt::HighDpiScaleFactorRoundingPolicy::PassThrough);

    QApplication app(*argc, argv);
    app.setOrganizationName(QStringLiteral("hdl-compose"));
    app.setApplicationName(QStringLiteral("HDL Compose"));
    app.setStyle(QStringLiteral("Fusion"));
    apply_material_dark_theme(app);

    MainWindow window;
    window.show();
    return app.exec();
}
