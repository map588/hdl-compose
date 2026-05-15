#include <QAction>
#include <QApplication>
#include <QCheckBox>
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

#include "hdl-compose/src/gui/bridge.cxxqt.h"

namespace {

constexpr char kModuleMimeType[] = "application/x-hdl-compose-module";
constexpr int kMinInstanceWidth = 200;
constexpr int kInstanceHeaderHeight = 50;
constexpr int kPinSlotHeight = 24;
constexpr int kPinShapeSize = 12;
constexpr int kMinInstanceBodyHeight = 30;
constexpr int kPinLabelHPadding = 8; // gap between pin tip and label start
constexpr int kInstanceCenterPadding = 24; // gap between left and right label columns
constexpr int kColumnPitch = 320; // X stride between adjacent column centers; gutter = pitch - max(width)
constexpr int kColumnGutterHalf = 60; // half-width of inter-column wire channel
constexpr int kWireLaneStep = 12; // X offset between adjacent wires sharing a gutter
constexpr int kClickThresholdPx = 5;
constexpr double kZoomMin = 0.2;
constexpr double kZoomMax = 5.0;
constexpr double kZoomStep = 1.15;
constexpr int kTopPortSpacing = 44;
constexpr int kWireStubMin = 40; // minimum pin-to-first-turn length (>= 4 × kPinShapeSize)
constexpr int kJunctionDotRadius = 4;

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

int find_instance_index(AppState *state, const QString &name) {
    int count = state->instance_count();
    for (int i = 0; i < count; ++i) {
        if (state->instance_name(i) == name) {
            return i;
        }
    }
    return -1;
}

QString allocate_instance_name(AppState *state, const QString &module) {
    int count = state->instance_count();
    QStringList existing;
    for (int i = 0; i < count; ++i) {
        existing << state->instance_name(i);
    }
    for (int i = 0;; ++i) {
        QString candidate = QStringLiteral("%1_%2").arg(module).arg(i);
        if (!existing.contains(candidate)) {
            return candidate;
        }
    }
}

// --- PortPinItem ------------------------------------------------------------

class InstanceItem;
class TopPortItem;
class PortPinItem;
class WireTool;
class CanvasLayer;

enum class PinSide { Left, Right };

class PortPinItem : public QGraphicsItem {
  public:
    PortPinItem(const QString &name, int direction, int width, PinSide side, InstanceItem *parent);
    ~PortPinItem() override;

    void setSlot(int slot_index) {
        m_slot = slot_index;
        prepareGeometryChange();
        update();
    }

    void setKey(const QString &k) { m_key = k; }
    QString key() const { return m_key; }
    int direction() const { return m_direction; }
    int width() const { return m_width; }
    QString portName() const { return m_name; }
    PinSide side() const { return m_side; }

    void setWireTool(WireTool *wt) { m_wire_tool = wt; }
    void setArmedState(bool armed) {
        if (m_armed_state != armed) {
            m_armed_state = armed;
            update();
        }
    }
    bool armedState() const { return m_armed_state; }

    virtual QPointF tipScenePos() const;

    void flashRed(int ms = 500);

    QRectF boundingRect() const override;

    // Hit-test on an 18 × 18 px region centered on the pin tip. Big enough to
    // click comfortably; small enough that clicks on the label row still fall
    // through to InstanceItem for drag/select.
    QPainterPath shape() const override;

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override;

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override;
    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override;
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override;

    QString m_name;
    QString m_key;   // "<inst>.<port>"
    int m_direction; // 0 in, 1 out, 2 inout
    int m_width;     // 0 scalar; N vector
    PinSide m_side;
    InstanceItem *m_parent;
    int m_slot = 0;
    WireTool *m_wire_tool = nullptr;
    bool m_flash = false;
    bool m_armed_state = false;
};

// --- BundlePinItem ---------------------------------------------------------

// BundlePinItem has two visual modes, chosen at layout time by parent InstanceItem:
//   - collapsed: fat shape representing the whole bundle; click expands.
//   - header:    arrow + name above the member pin list; click collapses.
// Member pin list is rendered as separate PortPinItems by the parent.
class BundlePinItem : public PortPinItem {
  public:
    BundlePinItem(const QString &bundle_name, PinSide side, bool header, int member_count, InstanceItem *parent)
        : PortPinItem(bundle_name, -1, 0, side, parent), m_header(header), m_member_count(member_count) {
        setAcceptedMouseButtons(Qt::LeftButton | Qt::RightButton);
    }

    QRectF boundingRect() const override;

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override;

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override;

  private:
    bool m_header;
    int m_member_count;
};

// --- InstanceItem -----------------------------------------------------------

class InstanceItem : public QGraphicsRectItem {
  public:
    InstanceItem(AppState *state, const QString &name, const QString &module, QGraphicsItem *parent = nullptr)
        : QGraphicsRectItem(0, 0, kMinInstanceWidth, kInstanceHeaderHeight + kMinInstanceBodyHeight, parent),
          m_state(state), m_name(name), m_module(module) {
        setFlags(QGraphicsItem::ItemIsMovable | QGraphicsItem::ItemIsSelectable |
                 QGraphicsItem::ItemSendsGeometryChanges);
        setAcceptedMouseButtons(Qt::LeftButton);
        layoutPins();
    }

    QString instanceName() const { return m_name; }
    AppState *state() const { return m_state; }
    void setInstanceName(const QString &n) { m_name = n; }
    void setModuleRef(const QString &m) { m_module = m; }
    int width() const { return m_width; }

    void setWireTool(WireTool *wt) {
        m_wire_tool = wt;
        for (auto *pin : m_pins) {
            pin->setWireTool(wt);
        }
    }

    void setCanvasLayer(CanvasLayer *layer) { m_canvas_layer = layer; }

    // Scene position of the anchor where a wire should attach for a given port name.
    // Falls back to the instance center if the port is not found.
    QPointF portAnchorScenePos(const QString &port_name) const {
        auto it = m_port_anchor.find(port_name);
        if (it == m_port_anchor.end()) {
            return mapToScene(rect().center());
        }
        return it.value()->tipScenePos();
    }

    void toggleBundleExpanded(const QString &bundle) {
        if (m_expanded_bundles.contains(bundle)) {
            m_expanded_bundles.remove(bundle);
        } else {
            m_expanded_bundles.insert(bundle);
        }
        relayoutPins();
    }

    bool bundleExpanded(const QString &bundle) const { return m_expanded_bundles.contains(bundle); }

    void relayoutPins() {
        for (auto *pin : m_pins) {
            scene()->removeItem(pin);
            delete pin;
        }
        m_pins.clear();
        layoutPins();
        update();
    }

    void layoutPins();

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *) override {
        QRectF r = rect();
        painter->setRenderHint(QPainter::Antialiasing);
        int idx = find_instance_index(m_state, m_name);
        bool dirty = (idx >= 0) && m_state->instance_is_dirty(idx);
        bool selected =
            (m_state->selected_instance() == m_name) || (option && (option->state & QStyle::State_Selected));

        // Body — flat dark fill, very subtle bottom panel under the header.
        painter->setPen(Qt::NoPen);
        painter->setBrush(QColor(36, 39, 44));
        painter->drawRoundedRect(r, 8, 8);

        // Header band — slightly lighter, drawn as a separate rounded rect
        // clipped to the top of the body so corner radii match.
        painter->save();
        QPainterPath body_path;
        body_path.addRoundedRect(r, 8, 8);
        painter->setClipPath(body_path);
        QRectF header_rect(r.left(), r.top(), r.width(), kInstanceHeaderHeight);
        painter->setBrush(QColor(46, 50, 56));
        painter->drawRect(header_rect);
        // Hairline divider under header
        painter->setPen(QColor(28, 30, 34));
        painter->drawLine(QPointF(r.left(), r.top() + kInstanceHeaderHeight),
                          QPointF(r.right(), r.top() + kInstanceHeaderHeight));
        painter->restore();

        // Border last so it sits over the fills.
        QPen pen;
        if (dirty) {
            pen.setColor(QColor(220, 90, 80));
            pen.setWidthF(selected ? 1.8 : 1.2);
        } else if (selected) {
            pen.setColor(QColor(120, 170, 240));
            pen.setWidthF(1.8);
        } else {
            pen.setColor(QColor(60, 64, 70));
            pen.setWidthF(1.0);
        }
        pen.setCosmetic(true);
        painter->setPen(pen);
        painter->setBrush(Qt::NoBrush);
        painter->drawRoundedRect(r, 8, 8);

        QFont name_font = painter->font();
        name_font.setBold(true);
        name_font.setPointSizeF(name_font.pointSizeF() + 0.5);
        painter->setFont(name_font);
        painter->setPen(QColor(232, 234, 238));
        painter->drawText(QRectF(r.left() + 12, r.top() + 8, r.width() - 24, 20),
                          Qt::AlignLeft | Qt::AlignVCenter, m_name);

        QFont mod_font = painter->font();
        mod_font.setBold(false);
        mod_font.setPointSizeF(mod_font.pointSizeF() - 1.5);
        painter->setFont(mod_font);
        painter->setPen(QColor(150, 154, 162));
        painter->drawText(QRectF(r.left() + 12, r.top() + 30, r.width() - 24, 16),
                          Qt::AlignLeft | Qt::AlignVCenter, m_module);
    }

  protected:
    void mousePressEvent(QGraphicsSceneMouseEvent *event) override {
        if (event->button() == Qt::LeftButton) {
            m_pressScenePos = event->scenePos();
            m_dragged = false;
        }
        QGraphicsRectItem::mousePressEvent(event);
    }

    void mouseMoveEvent(QGraphicsSceneMouseEvent *event) override {
        QGraphicsRectItem::mouseMoveEvent(event);
        if (!m_dragged) {
            QPointF delta = event->scenePos() - m_pressScenePos;
            if (delta.manhattanLength() >= kClickThresholdPx) {
                m_dragged = true;
            }
        }
    }

    void mouseReleaseEvent(QGraphicsSceneMouseEvent *event) override {
        QGraphicsRectItem::mouseReleaseEvent(event);
        if (event->button() != Qt::LeftButton) {
            return;
        }
        if (m_dragged) {
            QPointF p = pos();
            m_state->set_instance_position(m_name, p.x(), p.y());
        } else {
            m_state->set_selected_instance(m_name);
        }
        m_dragged = false;
    }

    QVariant itemChange(GraphicsItemChange change, const QVariant &value) override;

  private:
    AppState *m_state;
    QString m_name;
    QString m_module;
    QPointF m_pressScenePos;
    bool m_dragged = false;
    std::vector<PortPinItem *> m_pins;
    QSet<QString> m_expanded_bundles;
    QHash<QString, PortPinItem *> m_port_anchor;
    WireTool *m_wire_tool = nullptr;
    CanvasLayer *m_canvas_layer = nullptr;
    int m_width = kMinInstanceWidth;
};

// --- PortPinItem / BundlePinItem impls --------------------------------------

PortPinItem::PortPinItem(const QString &name, int direction, int width, PinSide side, InstanceItem *parent)
    : QGraphicsItem(parent), m_name(name), m_direction(direction), m_width(width), m_side(side), m_parent(parent) {
    setAcceptedMouseButtons(Qt::LeftButton);
}

QPointF PortPinItem::tipScenePos() const {
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    return mapToScene(QPointF(tip_x, y));
}

QRectF PortPinItem::boundingRect() const {
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal w = pw + 40;
    qreal x = (m_side == PinSide::Left) ? -kPinShapeSize - 4 : 0;
    return QRectF(x, m_slot * kPinSlotHeight, w, kPinSlotHeight);
}

QPainterPath PortPinItem::shape() const {
    QPainterPath p;
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    qreal half = 9.0;
    p.addRect(QRectF(tip_x - half, y - half, 2 * half, 2 * half));
    return p;
}

QRectF BundlePinItem::boundingRect() const {
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal w = pw + 40;
    qreal x = (m_side == PinSide::Left) ? -kPinShapeSize - 8 : 0;
    return QRectF(x, m_slot * kPinSlotHeight, w, kPinSlotHeight);
}

static QString format_width(int w) {
    if (w <= 0) {
        return QString();
    }
    return QStringLiteral("[%1:0]").arg(w - 1);
}

void PortPinItem::paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) {
    // Tip point at slot center, on the side edge.
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;

    // Draw shape
    painter->setRenderHint(QPainter::Antialiasing);

    // Armed-state halo: yellow ring around the pin tip when WireTool has armed us.
    if (m_armed_state) {
        painter->setBrush(Qt::NoBrush);
        painter->setPen(QPen(QColor(255, 215, 64), 2.5));
        painter->drawEllipse(QPointF(tip_x, y), kPinShapeSize + 2, kPinShapeSize + 2);
    }

    QColor pin_color(210, 210, 210);
    if (m_flash) {
        pin_color = QColor(232, 92, 80);
    } else if (m_direction == 0 /*In*/) {
        pin_color = QColor(120, 196, 152);
    } else if (m_direction == 1 /*Out*/) {
        pin_color = QColor(232, 168, 104);
    } else if (m_direction == 2 /*InOut*/) {
        pin_color = QColor(160, 174, 224);
    }
    painter->setBrush(pin_color);
    QPen pin_outline(QColor(20, 22, 26), 1);
    pin_outline.setCosmetic(true);
    painter->setPen(pin_outline);

    if (m_direction == 2) {
        // Diamond
        QPolygonF diamond;
        qreal half = kPinShapeSize / 2.0;
        diamond << QPointF(tip_x - half, y) << QPointF(tip_x, y - half) << QPointF(tip_x + half, y)
                << QPointF(tip_x, y + half);
        painter->drawPolygon(diamond);
    } else {
        // Triangle always points right (signal flows L→R). Tip is to the
        // right; base is on the inside of the module for inputs, on the
        // outside (extending past the module edge) for outputs.
        QPolygonF tri;
        if (m_side == PinSide::Left) {
            // Input on left edge: tip at module edge (tip_x), base outside-left.
            tri << QPointF(tip_x - kPinShapeSize, y - kPinShapeSize / 2.0) << QPointF(tip_x, y)
                << QPointF(tip_x - kPinShapeSize, y + kPinShapeSize / 2.0);
        } else {
            // Output on right edge: tip extends past module edge, base at edge.
            tri << QPointF(tip_x, y - kPinShapeSize / 2.0) << QPointF(tip_x + kPinShapeSize, y)
                << QPointF(tip_x, y + kPinShapeSize / 2.0);
        }
        painter->drawPolygon(tri);
    }

    // Label
    QString label = m_name;
    QString w = format_width(m_width);
    if (!w.isEmpty()) {
        label += w;
    }
    QFont f = painter->font();
    f.setPointSizeF(f.pointSizeF() - 1.0);
    if (!w.isEmpty()) {
        f.setFamily(QStringLiteral("Menlo"));
    }
    painter->setFont(f);
    painter->setPen(QColor(220, 220, 220));
    QRectF label_rect;
    if (m_side == PinSide::Left) {
        label_rect = QRectF(tip_x + 4, y - kPinSlotHeight / 2.0, pw - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignLeft | Qt::AlignVCenter, label);
    } else {
        label_rect = QRectF(tip_x - pw + 4, y - kPinSlotHeight / 2.0, pw - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignRight | Qt::AlignVCenter, label);
    }
    setToolTip(label);
}

void BundlePinItem::paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) {
    painter->setRenderHint(QPainter::Antialiasing);
    qreal y = m_slot * kPinSlotHeight + kPinSlotHeight / 2.0;
    qreal pw = m_parent ? m_parent->width() : kMinInstanceWidth;
    qreal tip_x = (m_side == PinSide::Left) ? 0 : pw;
    qreal half = kPinShapeSize;

    QString arrow = m_header ? QStringLiteral("▼") : QStringLiteral("▶");
    QString label = QStringLiteral("%1 %2 (%3)").arg(arrow, m_name).arg(m_member_count);

    if (!m_header) {
        // Collapsed: fat rounded rect on the edge.
        QRectF fat(tip_x - half, y - half, 2 * half, 2 * half);
        painter->setBrush(QColor(132, 162, 212));
        QPen ppen(QColor(20, 22, 26), 1.0);
        ppen.setCosmetic(true);
        painter->setPen(ppen);
        painter->drawRoundedRect(fat, 3, 3);
    } else {
        // Expanded header: thin horizontal bar spanning the side.
        QRectF bar;
        if (m_side == PinSide::Left) {
            bar = QRectF(tip_x, y - 1, 6, 2);
        } else {
            bar = QRectF(tip_x - 6, y - 1, 6, 2);
        }
        painter->setPen(Qt::NoPen);
        painter->setBrush(QColor(132, 162, 212));
        painter->drawRect(bar);
    }

    QFont f = painter->font();
    f.setBold(true);
    f.setPointSizeF(f.pointSizeF() - 1.0);
    painter->setFont(f);
    painter->setPen(QColor(m_header ? 180 : 235, m_header ? 200 : 235, m_header ? 220 : 235));
    QRectF label_rect;
    if (m_side == PinSide::Left) {
        label_rect = QRectF(tip_x + half + 4, y - kPinSlotHeight / 2.0, pw - half - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignLeft | Qt::AlignVCenter, label);
    } else {
        label_rect =
            QRectF(tip_x - pw + 4, y - kPinSlotHeight / 2.0, pw - half - 8, kPinSlotHeight);
        painter->drawText(label_rect, Qt::AlignRight | Qt::AlignVCenter, label);
    }
}

void BundlePinItem::mousePressEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_parent) {
        m_parent->toggleBundleExpanded(m_name);
        event->accept();
        return;
    }
    PortPinItem::mousePressEvent(event);
}

void InstanceItem::layoutPins() {
    m_port_anchor.clear();
    int idx = find_instance_index(m_state, m_name);
    if (idx < 0) {
        return;
    }
    int port_count = m_state->instance_port_count(idx);

    // Group ports by (side, bundle).
    // Build side ordered lists. Bundles collect their members.
    struct PortEntry {
        int port_index;
        QString name;
        int direction;
        int width;
        QString bundle;
    };
    // Build a reverse map port_name -> manual_bundle_name for this instance.
    // Manual bundles replace any auto-detected `PortDef.bundle` value.
    QHash<QString, QString> manual_bundle_of;
    {
        int bc = m_state->manual_bundle_count(idx);
        for (int b = 0; b < bc; ++b) {
            QString bname = m_state->manual_bundle_name(idx, b);
            int pc = m_state->manual_bundle_port_count(idx, b);
            for (int pp = 0; pp < pc; ++pp) {
                manual_bundle_of.insert(m_state->manual_bundle_port_name(idx, b, pp), bname);
            }
        }
    }

    std::vector<PortEntry> entries;
    entries.reserve(port_count);
    for (int p = 0; p < port_count; ++p) {
        QString port_name = m_state->instance_port_name(idx, p);
        QString bundle = manual_bundle_of.value(port_name, m_state->instance_port_bundle(idx, p));
        entries.push_back(
            {p, port_name, m_state->instance_port_direction(idx, p), m_state->instance_port_width(idx, p), bundle});
    }

    // Direction → side. InOut → Left (convention).
    auto side_for = [](int dir) { return (dir == 1) ? PinSide::Right : PinSide::Left; };

    // Bundle side = majority direction of members. Ties → Left (convention).
    // Two-pass: first count outs vs non-outs per bundle, then place all
    // members on the chosen side regardless of individual direction.
    QMap<QString, int> bundle_out_count;
    QMap<QString, int> bundle_total;
    for (const auto &e : entries) {
        if (e.bundle.isEmpty())
            continue;
        bundle_total[e.bundle] = bundle_total.value(e.bundle, 0) + 1;
        if (e.direction == 1)
            bundle_out_count[e.bundle] = bundle_out_count.value(e.bundle, 0) + 1;
    }
    auto bundle_side = [&](const QString &bname) {
        int outs = bundle_out_count.value(bname, 0);
        int total = bundle_total.value(bname, 0);
        return (outs * 2 > total) ? PinSide::Right : PinSide::Left;
    };

    // QMap keeps bundles sorted by name for stable layout.
    QMap<QString, std::vector<int>> left_bundles;
    QMap<QString, std::vector<int>> right_bundles;
    std::vector<int> left_ports, right_ports;
    for (const auto &e : entries) {
        if (!e.bundle.isEmpty()) {
            auto side = bundle_side(e.bundle);
            auto &map = (side == PinSide::Left) ? left_bundles : right_bundles;
            map[e.bundle].push_back(e.port_index);
        } else {
            auto side = side_for(e.direction);
            auto &list = (side == PinSide::Left) ? left_ports : right_ports;
            list.push_back(e.port_index);
        }
    }

    int left_slot = 0;
    int right_slot = 0;

    auto add_port_pin = [&](int port_idx, PinSide side, int &slot) {
        const auto &e = entries[port_idx];
        auto *pin = new PortPinItem(e.name, e.direction, e.width, side, this);
        pin->setSlot(slot);
        pin->setPos(0, kInstanceHeaderHeight);
        pin->setKey(QStringLiteral("%1.%2").arg(m_name, e.name));
        pin->setWireTool(m_wire_tool);
        m_pins.push_back(pin);
        m_port_anchor.insert(e.name, pin);
        slot++;
    };
    auto add_bundle_group = [&](const QString &bname, const std::vector<int> &members, PinSide side, int &slot) {
        bool expanded = bundleExpanded(bname);
        auto *header = new BundlePinItem(bname, side, expanded, static_cast<int>(members.size()), this);
        header->setSlot(slot);
        header->setPos(0, kInstanceHeaderHeight);
        m_pins.push_back(header);
        // Collapsed: all member ports anchor to the bundle header.
        if (!expanded) {
            for (int p : members) {
                m_port_anchor.insert(entries[p].name, header);
            }
        }
        slot++;
        if (expanded) {
            for (int p : members) {
                add_port_pin(p, side, slot);
            }
        }
    };

    for (int p : left_ports)
        add_port_pin(p, PinSide::Left, left_slot);
    for (auto it = left_bundles.constBegin(); it != left_bundles.constEnd(); ++it) {
        add_bundle_group(it.key(), it.value(), PinSide::Left, left_slot);
    }
    for (int p : right_ports)
        add_port_pin(p, PinSide::Right, right_slot);
    for (auto it = right_bundles.constBegin(); it != right_bundles.constEnd(); ++it) {
        add_bundle_group(it.key(), it.value(), PinSide::Right, right_slot);
    }

    int slot_total = (left_slot > right_slot) ? left_slot : right_slot;
    int pin_body = slot_total * kPinSlotHeight + 8;
    int body = (pin_body > kMinInstanceBodyHeight) ? pin_body : kMinInstanceBodyHeight;

    // Width = max label column on each side + gap. Use a slightly-smaller
    // font than default to mirror PortPinItem::paint sizing; bold for the
    // header so the title row's reservation doesn't wrap.
    QFont label_font;
    label_font.setPointSizeF(label_font.pointSizeF() - 1.0);
    QFontMetrics fm(label_font);
    QFont header_font;
    header_font.setBold(true);
    header_font.setPointSizeF(header_font.pointSizeF() + 1.0);
    QFontMetrics hf(header_font);

    auto label_for = [&](const PortEntry &e, bool is_bundle_header, int member_count) {
        if (is_bundle_header) {
            return QStringLiteral("▼ %1 (%2)").arg(e.bundle).arg(member_count);
        }
        QString s = e.name;
        if (e.width > 0) {
            s += QStringLiteral("[%1:0]").arg(e.width - 1);
        }
        return s;
    };
    auto pin_label_w = [&](const PortEntry &e) {
        return fm.horizontalAdvance(label_for(e, false, 0));
    };

    int left_label_w = 0;
    for (int p : left_ports) {
        left_label_w = std::max(left_label_w, pin_label_w(entries[p]));
    }
    for (auto it = left_bundles.constBegin(); it != left_bundles.constEnd(); ++it) {
        int hw = fm.horizontalAdvance(QStringLiteral("▼ %1 (%2)").arg(it.key()).arg(static_cast<int>(it.value().size())));
        left_label_w = std::max(left_label_w, hw);
        if (bundleExpanded(it.key())) {
            for (int p : it.value()) {
                left_label_w = std::max(left_label_w, pin_label_w(entries[p]));
            }
        }
    }
    int right_label_w = 0;
    for (int p : right_ports) {
        right_label_w = std::max(right_label_w, pin_label_w(entries[p]));
    }
    for (auto it = right_bundles.constBegin(); it != right_bundles.constEnd(); ++it) {
        int hw = fm.horizontalAdvance(QStringLiteral("▼ %1 (%2)").arg(it.key()).arg(static_cast<int>(it.value().size())));
        right_label_w = std::max(right_label_w, hw);
        if (bundleExpanded(it.key())) {
            for (int p : it.value()) {
                right_label_w = std::max(right_label_w, pin_label_w(entries[p]));
            }
        }
    }
    int header_w = hf.horizontalAdvance(QStringLiteral("%1 : %2").arg(m_name, m_module)) + 24;
    int needed = left_label_w + right_label_w + kPinShapeSize * 2 + kInstanceCenterPadding + kPinLabelHPadding * 2;
    needed = std::max(needed, header_w);
    m_width = std::max(needed, kMinInstanceWidth);

    setRect(0, 0, m_width, kInstanceHeaderHeight + body);
}

// --- WireTool + WireItem ----------------------------------------------------

class WireItem;

class WireTool {
  public:
    WireTool(AppState *state, QGraphicsScene *scene) : m_state(state), m_scene(scene) {}

    // Press on a pin: either commits (if click-click mid-flight) or arms+starts drag.
    void onPinPressed(PortPinItem *pin, const QPointF &scene_pos);
    // During a held drag, update the provisional wire endpoint.
    void onPinDragMove(const QPointF &scene_pos);
    // Release: commit if over a pin, keep armed if movement was tiny, else cancel.
    void onPinReleased(const QPointF &scene_pos);

    void cancel();
    void notifyPinDestroyed(PortPinItem *pin) {
        if (m_armed == pin)
            m_armed = nullptr;
    }
    PortPinItem *armed() const { return m_armed; }

  private:
    // Compatibility check. Returns empty string if compatible, else reason.
    QString compatibilityError(PortPinItem *src, PortPinItem *dst) const;
    // Attempt to commit src→dst; flash-red if incompatible. Returns true on commit.
    bool tryCommit(PortPinItem *src, PortPinItem *dst);
    // Input↔input driver-resolution flow for multi-load nets.
    bool tryCommitMultiLoad(PortPinItem *a, PortPinItem *b);
    void createProvisional(PortPinItem *from, const QPointF &scene_pos);
    void clearProvisional();

    AppState *m_state;
    QGraphicsScene *m_scene;
    PortPinItem *m_armed = nullptr;
    QGraphicsPathItem *m_provisional = nullptr;
    QPointF m_press_pos;
    // Set by tryCommitMultiLoad on a successful multi-load commit; instructs
    // onPinReleased to keep m_armed alive so the user can click more pins to
    // keep extending the same net.
    bool m_sticky_after_commit = false;
};

class WireItem : public QGraphicsPathItem {
  public:
    WireItem(const QString &source_key, const QString &target_key)
        : m_source_key(source_key), m_target_key(target_key) {
        setFlag(QGraphicsItem::ItemIsSelectable, true);
        setAcceptedMouseButtons(Qt::LeftButton | Qt::RightButton);
        QPen pen(colorForNet(source_key), 1.5);
        pen.setCosmetic(true);
        setPen(pen);
    }

    // Hash the net identity to a deterministic hue. Same net → same color;
    // different nets land on clearly distinct hues so co-routed wires
    // (e.g. clk and rst_n bundled toward the same instance) are
    // distinguishable at a glance.
    static QColor colorForNet(const QString &key) {
        // Muted palette: lower saturation + slightly dimmer value so wires
        // read as background detail rather than competing with module bodies.
        int hue = static_cast<int>(qHash(key) % 360);
        return QColor::fromHsv(hue, 110, 200);
    }

    const QString &sourceKey() const { return m_source_key; }
    const QString &targetKey() const { return m_target_key; }

    void setRouteIndex(int idx) { m_route_index = idx; }

    // Apply a precomputed manhattan path. Caller (CanvasLayer) plans waypoints
    // with full knowledge of all modules + gutter lane assignment.
    void setWaypoints(const QVector<QPointF> &pts) {
        m_waypoints = pts;
        QPainterPath p;
        if (!pts.isEmpty()) {
            p.moveTo(pts.first());
            for (int i = 1; i < pts.size(); ++i)
                p.lineTo(pts[i]);
        }
        setPath(p);
    }

    const QVector<QPointF> &waypoints() const { return m_waypoints; }

    // Wider hit region so cosmetic-pen wires can actually be clicked.
    QPainterPath shape() const override {
        QPainterPathStroker stroker;
        stroker.setWidth(10.0);
        return stroker.createStroke(path());
    }

    // Inflate the path bbox so the bus-width slash + label (drawn ~9 px
    // perpendicular to the segment) stays inside the repaint area. Without
    // this, dragging a wire leaves stale label artifacts on screen.
    QRectF boundingRect() const override { return QGraphicsPathItem::boundingRect().adjusted(-16, -16, 16, 16); }

    void setAppState(AppState *s) { m_state = s; }
    void setWidth(int w) { m_width = w; }

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *option, QWidget *widget) override {
        // Render selected state in a distinct high-contrast color.
        if (option->state & QStyle::State_Selected) {
            QPen sel_pen(QColor(66, 180, 244), 2.0);
            sel_pen.setCosmetic(true);
            painter->setPen(sel_pen);
            painter->drawPath(path());
        } else {
            QGraphicsPathItem::paint(painter, option, widget);
        }

        // Bus annotation: short slash + width number on multi-bit wires.
        // VHDL/Verilog schematic convention — single line with `--/--` cap and
        // the bit count above. Skipped for scalar (width 1) and unknown (-1).
        if (m_width <= 1)
            return;
        const QPainterPath p = path();
        if (p.elementCount() < 2)
            return;
        // Pick the longest segment so the annotation lands somewhere visible.
        qreal best_len = 0;
        QPointF a, b;
        for (int i = 1; i < p.elementCount(); ++i) {
            QPointF p0(p.elementAt(i - 1).x, p.elementAt(i - 1).y);
            QPointF p1(p.elementAt(i).x, p.elementAt(i).y);
            qreal dx = p1.x() - p0.x(), dy = p1.y() - p0.y();
            qreal len = std::sqrt(dx * dx + dy * dy);
            if (len > best_len) {
                best_len = len;
                a = p0;
                b = p1;
            }
        }
        if (best_len < 16.0)
            return;
        QPointF mid((a.x() + b.x()) / 2.0, (a.y() + b.y()) / 2.0);
        // Slash perpendicular to the segment, centered on mid.
        QPointF dir(b.x() - a.x(), b.y() - a.y());
        qreal dlen = std::sqrt(dir.x() * dir.x() + dir.y() * dir.y());
        QPointF perp(-dir.y() / dlen, dir.x() / dlen);
        constexpr qreal kSlashHalf = 5.0;
        QPointF s1(mid.x() + perp.x() * kSlashHalf - dir.x() / dlen * kSlashHalf,
                   mid.y() + perp.y() * kSlashHalf - dir.y() / dlen * kSlashHalf);
        QPointF s2(mid.x() - perp.x() * kSlashHalf + dir.x() / dlen * kSlashHalf,
                   mid.y() - perp.y() * kSlashHalf + dir.y() / dlen * kSlashHalf);
        QPen slash_pen(colorForNet(m_source_key), 1.5);
        slash_pen.setCosmetic(true);
        painter->setPen(slash_pen);
        painter->drawLine(s1, s2);
        // Width number above the slash on the perp side.
        painter->setPen(QColor(220, 220, 220));
        QFont f = painter->font();
        f.setPointSizeF(9.0);
        painter->setFont(f);
        QPointF label_pt(mid.x() + perp.x() * 9.0 - 4.0, mid.y() + perp.y() * 9.0 + 4.0);
        painter->drawText(label_pt, QString::number(m_width));
    }

  protected:
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override;

  private:
    QString m_source_key;
    QString m_target_key;
    AppState *m_state = nullptr;
    int m_width = 1;
    int m_route_index = -1;
    QVector<QPointF> m_waypoints;
};

// Filled circle marking a wire junction (T-tap of same-net wires).
class JunctionDotItem : public QGraphicsEllipseItem {
  public:
    JunctionDotItem(const QPointF &center, const QColor &color)
        : QGraphicsEllipseItem(center.x() - kJunctionDotRadius, center.y() - kJunctionDotRadius,
                               2 * kJunctionDotRadius, 2 * kJunctionDotRadius) {
        setBrush(color);
        QPen pen(color);
        pen.setCosmetic(true);
        setPen(pen);
        setZValue(2);
    }
};

PortPinItem::~PortPinItem() {
    if (m_wire_tool) {
        m_wire_tool->notifyPinDestroyed(this);
    }
}

void PortPinItem::flashRed(int ms) {
    m_flash = true;
    update();
    QTimer::singleShot(ms, [this]() {
        m_flash = false;
        update();
    });
}

void PortPinItem::mousePressEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_wire_tool) {
        m_wire_tool->onPinPressed(this, event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mousePressEvent(event);
}

void PortPinItem::mouseMoveEvent(QGraphicsSceneMouseEvent *event) {
    if ((event->buttons() & Qt::LeftButton) && m_wire_tool) {
        m_wire_tool->onPinDragMove(event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mouseMoveEvent(event);
}

void PortPinItem::mouseReleaseEvent(QGraphicsSceneMouseEvent *event) {
    if (event->button() == Qt::LeftButton && m_wire_tool) {
        m_wire_tool->onPinReleased(event->scenePos());
        event->accept();
        return;
    }
    QGraphicsItem::mouseReleaseEvent(event);
}

// Prompt the user to create or edit a manual bundle on `instance`. If
// `preselected_port` is non-empty it starts checked. Returns true on success.
static bool prompt_create_manual_bundle(QWidget *parent, AppState *state, const QString &instance,
                                        const QString &preselected_port) {
    int idx = find_instance_index(state, instance);
    if (idx < 0)
        return false;
    int n = state->instance_port_count(idx);
    if (n == 0)
        return false;

    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("Group into interface"));

    auto *name_edit = new QLineEdit(&dlg);
    name_edit->setPlaceholderText(QStringLiteral("e.g. spi, m_axi, s_axi_lite"));

    auto *box_layout = new QVBoxLayout;
    QList<QCheckBox *> checks;
    checks.reserve(n);
    for (int p = 0; p < n; ++p) {
        QString pname = state->instance_port_name(idx, p);
        auto *cb = new QCheckBox(pname, &dlg);
        if (pname == preselected_port)
            cb->setChecked(true);
        box_layout->addWidget(cb);
        checks.append(cb);
    }

    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);

    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("&Bundle name:"), name_edit);

    auto *scroll_inner = new QWidget;
    scroll_inner->setLayout(box_layout);
    auto *scroll = new QScrollArea;
    scroll->setWidget(scroll_inner);
    scroll->setWidgetResizable(true);
    scroll->setMinimumHeight(200);

    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(new QLabel(QStringLiteral("Ports:"), &dlg));
    layout->addWidget(scroll);
    layout->addWidget(buttons);

    if (dlg.exec() != QDialog::Accepted)
        return false;
    QString name = name_edit->text().trimmed();
    if (name.isEmpty())
        return false;
    QStringList picked;
    for (auto *cb : checks) {
        if (cb->isChecked())
            picked << cb->text();
    }
    if (picked.size() < 2)
        return false;
    return state->create_manual_bundle(instance, name, picked.join(QChar(',')));
}

// Dialog: enter a driver expression + slice spec for a multi-bit port.
// Returns true if a slice was applied.
static bool prompt_connect_slice(QWidget *parent, AppState *state, const QString &instance, const QString &port) {
    QDialog dlg(parent);
    dlg.setWindowTitle(QStringLiteral("Connect slice"));
    auto *driver_edit = new QLineEdit(&dlg);
    driver_edit->setPlaceholderText(QStringLiteral("driver (e.g. u_counter.count, or clk for a top-port)"));
    auto *slice_edit = new QLineEdit(&dlg);
    slice_edit->setPlaceholderText(QStringLiteral("slice (e.g. 0 or 7:4)"));
    auto *form = new QFormLayout;
    form->addRow(QStringLiteral("Driver:"), driver_edit);
    form->addRow(QStringLiteral("Slice:"), slice_edit);
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, &dlg);
    QObject::connect(buttons, &QDialogButtonBox::accepted, &dlg, &QDialog::accept);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &dlg, &QDialog::reject);
    auto *layout = new QVBoxLayout(&dlg);
    layout->addLayout(form);
    layout->addWidget(buttons);
    if (dlg.exec() != QDialog::Accepted)
        return false;

    QString driver = driver_edit->text().trimmed();
    QString slice = slice_edit->text().trimmed();
    slice.remove(QChar('[')).remove(QChar(']'));
    if (driver.isEmpty() || slice.isEmpty())
        return false;

    // driver_inst / driver_port split on first '.'
    QString driver_inst, driver_port;
    int dot = driver.indexOf(QChar('.'));
    if (dot < 0) {
        driver_inst = QString(); // top-port
        driver_port = driver;
    } else {
        driver_inst = driver.left(dot);
        driver_port = driver.mid(dot + 1);
    }

    // Parse slice: "h:l" or single integer.
    int high = 0, low = 0;
    int colon = slice.indexOf(QChar(':'));
    bool ok = false;
    if (colon < 0) {
        int v = slice.toInt(&ok);
        if (!ok)
            return false;
        high = v;
        low = v;
    } else {
        high = slice.left(colon).trimmed().toInt(&ok);
        if (!ok)
            return false;
        low = slice.mid(colon + 1).trimmed().toInt(&ok);
        if (!ok)
            return false;
    }
    return state->set_port_map_entry_slice(instance, port, driver_inst, driver_port, high, low);
}

void PortPinItem::contextMenuEvent(QGraphicsSceneContextMenuEvent *event) {
    if (!m_parent || !m_parent->state()) {
        return;
    }
    AppState *state = m_parent->state();
    QString inst = m_parent->instanceName();
    bool is_bundle = dynamic_cast<BundlePinItem *>(this) != nullptr;

    QMenu menu;
    QAction *groupAct = nullptr;
    QAction *ungroupAct = nullptr;
    QAction *promoteAct = nullptr;
    QAction *clearAct = nullptr;
    QAction *sliceAct = nullptr;
    if (is_bundle) {
        ungroupAct = menu.addAction(QStringLiteral("Ungroup"));
    } else {
        groupAct = menu.addAction(QStringLiteral("Group into interface..."));
        promoteAct = menu.addAction(QStringLiteral("Promote to top-level port"));
        // Slice dialog makes sense only for multi-bit ports (width != 0).
        if (m_width != 0) {
            sliceAct = menu.addAction(QStringLiteral("Connect slice..."));
        }
        clearAct = menu.addAction(QStringLiteral("Clear connection"));
    }
    QAction *chosen = menu.exec(event->screenPos());
    if (!chosen) {
        event->accept();
        return;
    }
    if (chosen == promoteAct) {
        QString resolved = state->promote_port_to_top(inst, m_name);
        if (!resolved.isEmpty() && resolved != m_name) {
            QToolTip::showText(QCursor::pos(), QStringLiteral("Promoted as '%1'").arg(resolved));
        }
    } else if (chosen == groupAct) {
        QWidget *parent_w = nullptr;
        if (auto *s = scene()) {
            if (!s->views().isEmpty())
                parent_w = s->views().first()->window();
        }
        prompt_create_manual_bundle(parent_w, state, inst, m_name);
    } else if (chosen == ungroupAct) {
        state->remove_manual_bundle(inst, m_name);
    } else if (chosen == clearAct) {
        state->clear_port_map_entry(inst, m_name);
    } else if (chosen == sliceAct) {
        QWidget *parent_w = nullptr;
        if (auto *s = scene()) {
            if (!s->views().isEmpty())
                parent_w = s->views().first()->window();
        }
        prompt_connect_slice(parent_w, state, inst, m_name);
    }
    event->accept();
}

// --- TopPortItem ------------------------------------------------------------

// Top-level port on the scene boundary. Inherits PortPinItem so it participates
// in WireTool dragging/clicking exactly like an instance pin.
class TopPortItem : public PortPinItem {
  public:
    TopPortItem(const QString &name, int direction, int width, PinSide side)
        : PortPinItem(name, direction, width, side, /*parent*/ nullptr) {
        setKey(QStringLiteral("top:%1").arg(name));
        setAcceptedMouseButtons(Qt::LeftButton);
    }

    QPointF tipScenePos() const override { return mapToScene(QPointF(0, 0)); }

    // Tight hit region around the chevron tip (18 × 18 px, matching PortPinItem).
    QPainterPath shape() const override {
        QPainterPath p;
        qreal half = 9.0;
        p.addRect(QRectF(-half, -half, 2 * half, 2 * half));
        return p;
    }

    // Right-click on a top-port: no menu yet (rename alias can land later).
    void contextMenuEvent(QGraphicsSceneContextMenuEvent *event) override { event->accept(); }

    void paint(QPainter *painter, const QStyleOptionGraphicsItem *, QWidget *) override {
        painter->setRenderHint(QPainter::Antialiasing);

        // Armed-state halo around the chevron tip.
        if (armedState()) {
            painter->setBrush(Qt::NoBrush);
            painter->setPen(QPen(QColor(255, 215, 64), 2.5));
            painter->drawEllipse(QPointF(0, 0), kPinShapeSize + 2, kPinShapeSize + 2);
        }

        QColor c = (direction() == 0) ? QColor(140, 200, 140) : QColor(200, 160, 110);
        painter->setBrush(c);
        painter->setPen(QPen(QColor(30, 30, 30), 1));
        // Top-level chevrons always point right. Wire connects at (0,0) which
        // is the canvas-edge anchor. Base of the triangle sits "outside" the
        // canvas (to the left for inputs, to the right past the tip for
        // outputs) so the chevron flows naturally into the wire.
        QPolygonF poly;
        if (side() == PinSide::Left) {
            // Input: tip at (0,0) (where wire starts), base to the left.
            poly << QPointF(-kPinShapeSize, -kPinShapeSize / 2.0) << QPointF(0, 0)
                 << QPointF(-kPinShapeSize, kPinShapeSize / 2.0);
        } else {
            // Output: base at (0,0) (where wire ends), tip extends right.
            poly << QPointF(0, -kPinShapeSize / 2.0) << QPointF(kPinShapeSize, 0) << QPointF(0, kPinShapeSize / 2.0);
        }
        painter->drawPolygon(poly);

        QString label = portName();
        QString w = format_width(width());
        if (!w.isEmpty())
            label += w;
        QFont f = painter->font();
        f.setBold(true);
        painter->setFont(f);
        painter->setPen(QColor(220, 220, 220));
        // Label sits OUTSIDE the canvas — left of inputs, right of outputs —
        // so it never overlaps the wire that runs from the chevron into the
        // design.
        if (side() == PinSide::Left) {
            painter->drawText(QRectF(-180 - kPinShapeSize - 6, -kPinSlotHeight / 2.0, 180, kPinSlotHeight),
                              Qt::AlignRight | Qt::AlignVCenter, label);
        } else {
            painter->drawText(QRectF(kPinShapeSize + 6, -kPinSlotHeight / 2.0, 180, kPinSlotHeight),
                              Qt::AlignLeft | Qt::AlignVCenter, label);
        }
    }

    QRectF boundingRect() const override {
        // Match the painter's rectangle (label extends 180px outward from the
        // chevron base, plus the chevron itself out to ±kPinShapeSize).
        if (side() == PinSide::Left) {
            return QRectF(-180 - kPinShapeSize - 6, -kPinSlotHeight / 2.0, 180 + kPinShapeSize + 6, kPinSlotHeight);
        }
        return QRectF(0, -kPinSlotHeight / 2.0, kPinShapeSize + 6 + 180, kPinSlotHeight);
    }
};

// --- WireTool impl ----------------------------------------------------------

QString WireTool::compatibilityError(PortPinItem *src, PortPinItem *dst) const {
    if (src == dst) {
        return QStringLiteral("cannot connect pin to itself");
    }
    bool src_top = src->key().startsWith(QStringLiteral("top:"));
    bool dst_top = dst->key().startsWith(QStringLiteral("top:"));
    // Two instance outputs: real multi-driver error. Top-ports have flipped
    // driver/load semantics (top-output is a load, top-input is a driver), so
    // any pair with a top-port involved passes through the direction check.
    if (!src_top && !dst_top && src->direction() == 1 && dst->direction() == 1) {
        return QStringLiteral("output-to-output: only one driver per net allowed");
    }
    int sw = src->width();
    int dw = dst->width();
    // Width-1 vector ≡ scalar for connection purposes (single wire either
    // way). Normalize so std_logic ↔ std_logic_vector(0 downto 0) connects
    // cleanly; codegen handles the index when it emits the port_map.
    if (sw == 1)
        sw = 0;
    if (dw == 1)
        dw = 0;
    // Width semantics: 0 = scalar (or width-1 vec), N>1 = resolved vector,
    // -1 = unresolved-width vector (generic-sized). Two unresolved vectors
    // pass through (cannot prove mismatch without generic eval).
    if (sw == 0 && dw == -1) {
        return QStringLiteral("type mismatch: scalar cannot drive vector");
    }
    if (sw == -1 && dw == 0) {
        return QStringLiteral("type mismatch: vector cannot drive scalar");
    }
    if (sw >= 0 && dw >= 0 && sw != dw) {
        return QStringLiteral("width mismatch: %1 vs %2")
            .arg(sw == 0 ? QStringLiteral("scalar") : QString::number(sw))
            .arg(dw == 0 ? QStringLiteral("scalar") : QString::number(dw));
    }
    return QString();
}

void WireTool::cancel() {
    clearProvisional();
    if (m_armed) {
        PortPinItem *prev = m_armed;
        m_armed = nullptr;
        prev->setArmedState(false);
    }
}

void WireTool::clearProvisional() {
    if (m_provisional) {
        m_scene->removeItem(m_provisional);
        delete m_provisional;
        m_provisional = nullptr;
    }
}

void WireTool::createProvisional(PortPinItem *from, const QPointF &scene_pos) {
    clearProvisional();
    m_provisional = new QGraphicsPathItem();
    QPen pen(QColor(255, 215, 64, 180), 1.5);
    pen.setCosmetic(true);
    pen.setStyle(Qt::DashLine);
    m_provisional->setPen(pen);
    m_provisional->setZValue(1000);
    QPainterPath p;
    p.moveTo(from->tipScenePos());
    p.lineTo(scene_pos);
    m_provisional->setPath(p);
    m_scene->addItem(m_provisional);
}

bool WireTool::tryCommit(PortPinItem *src, PortPinItem *dst) {
    QString err = compatibilityError(src, dst);
    if (!err.isEmpty()) {
        dst->flashRed();
        QToolTip::showText(QCursor::pos(), err);
        return false;
    }
    bool src_top = src->key().startsWith(QStringLiteral("top:"));
    bool dst_top = dst->key().startsWith(QStringLiteral("top:"));

    // Top-port ↔ top-port: not modelled (net identity is a single top-port).
    if (src_top && dst_top) {
        QToolTip::showText(QCursor::pos(), QStringLiteral("cannot wire two top-level ports"));
        return false;
    }

    // Top-port ↔ instance-port: direct assignment. The top-port is the net
    // identity regardless of direction; the instance side records the
    // connection in its port_map.
    if (src_top || dst_top) {
        PortPinItem *top = src_top ? src : dst;
        PortPinItem *inst_pin = src_top ? dst : src;
        QString inst_key = inst_pin->key();
        int dot = inst_key.indexOf(QChar('.'));
        if (dot < 0)
            return false;
        QString inst = inst_key.left(dot);
        QString port = inst_key.mid(dot + 1);
        QString top_name = top->key().mid(4); // strip "top:"
        m_state->set_port_map_entry(inst, port, top_name);
        return true;
    }

    // Both instance-pins: input↔input uses multi-load driver resolution.
    if (src->direction() == 0 && dst->direction() == 0) {
        return tryCommitMultiLoad(src, dst);
    }
    auto can_drive = [](int dir) { return dir == 1 || dir == 2; };
    PortPinItem *driver = can_drive(src->direction()) ? src : dst;
    PortPinItem *load = (driver == src) ? dst : src;
    QString load_key = load->key();
    int dot = load_key.indexOf(QChar('.'));
    if (dot < 0)
        return false;
    QString inst = load_key.left(dot);
    QString port = load_key.mid(dot + 1);
    QString dkey = driver->key();
    QString driver_rhs = dkey.startsWith(QStringLiteral("top:")) ? dkey.mid(4) : dkey;
    // std_logic_vector(0 downto 0) driving std_logic: VHDL needs an explicit
    // index on the vector side. Store as a Bit(0) slice so codegen emits
    // `<load> => <driver>(0)`.
    if (driver->width() == 1 && load->width() == 0) {
        int dd = dkey.indexOf(QChar('.'));
        if (dd > 0 && !dkey.startsWith(QStringLiteral("top:"))) {
            QString d_inst = dkey.left(dd);
            QString d_port = dkey.mid(dd + 1);
            m_state->set_port_map_entry_slice(inst, port, d_inst, d_port, 0, 0);
            return true;
        }
    }
    m_state->set_port_map_entry(inst, port, driver_rhs);
    return true;
}

static bool parse_pin_key(PortPinItem *pin, QString *inst, QString *port) {
    QString k = pin->key();
    int dot = k.indexOf(QChar('.'));
    if (dot < 0)
        return false;
    *inst = k.left(dot);
    *port = k.mid(dot + 1);
    return true;
}

bool WireTool::tryCommitMultiLoad(PortPinItem *a, PortPinItem *b) {
    QString a_inst, a_port, b_inst, b_port;
    if (!parse_pin_key(a, &a_inst, &a_port))
        return false;
    if (!parse_pin_key(b, &b_inst, &b_port))
        return false;

    QString a_rhs = m_state->port_map_entry(a_inst, a_port);
    QString b_rhs = m_state->port_map_entry(b_inst, b_port);
    bool a_driven = !a_rhs.isEmpty();
    bool b_driven = !b_rhs.isEmpty();

    // One side already driven — extend that net to the other pin.
    if (a_driven != b_driven) {
        const QString &rhs = a_driven ? a_rhs : b_rhs;
        const QString &target_inst = a_driven ? b_inst : a_inst;
        const QString &target_port = a_driven ? b_port : a_port;
        return m_state->set_port_map_entry(target_inst, target_port, rhs);
    }
    // Both already driven from the same net — no-op.
    if (a_driven && b_driven && a_rhs == b_rhs) {
        return true;
    }
    // Both driven from different nets — silently route B onto A's net.
    // Neither driven — create an undriven internal signal whose identity is
    // pin A. Both pins' port_map references that identity; codegen emits a
    // signal declaration and both ports wire to it. A driver can be added
    // later with a separate wire.
    QString rhs = QStringLiteral("%1.%2").arg(a_inst, a_port);
    m_state->set_port_map_entry(a_inst, a_port, rhs);
    m_state->set_port_map_entry(b_inst, b_port, rhs);
    // Net-sharing intent: keep pin A armed after commit so the user can click
    // more input pins to add them to the same net. Esc or re-clicking A cancels.
    m_sticky_after_commit = true;
    return true;
}

void WireTool::onPinPressed(PortPinItem *pin, const QPointF &scene_pos) {
    if (!pin)
        return;
    m_press_pos = scene_pos;
    if (m_armed && m_armed != pin) {
        // Click-click commit attempt — finish the pending wire, no new provisional.
        tryCommit(m_armed, pin);
        if (m_sticky_after_commit) {
            m_sticky_after_commit = false;
            clearProvisional();
            return; // keep armed for net-building
        }
        cancel();
        return;
    }
    if (m_armed == pin) {
        // Click-on-armed-pin cancels.
        cancel();
        return;
    }
    // Arm this pin and start a provisional wire from its tip to the cursor.
    m_armed = pin;
    pin->setArmedState(true);
    createProvisional(pin, scene_pos);
}

void WireTool::onPinDragMove(const QPointF &scene_pos) {
    if (!m_armed || !m_provisional)
        return;
    QPainterPath p;
    p.moveTo(m_armed->tipScenePos());
    p.lineTo(scene_pos);
    m_provisional->setPath(p);
}

void WireTool::onPinReleased(const QPointF &scene_pos) {
    if (!m_armed)
        return;
    // Check what's at the release point.
    PortPinItem *target = nullptr;
    for (QGraphicsItem *it : m_scene->items(scene_pos)) {
        if (auto *pin = dynamic_cast<PortPinItem *>(it)) {
            if (pin != m_armed) {
                target = pin;
                break;
            }
        }
    }
    if (target) {
        tryCommit(m_armed, target);
        if (m_sticky_after_commit) {
            m_sticky_after_commit = false;
            // Multi-load commit: keep pin armed for continued net-building.
            clearProvisional();
            return;
        }
        cancel();
        return;
    }
    // No target pin. Short press with minimal movement = keep armed for click-click.
    if ((scene_pos - m_press_pos).manhattanLength() < kClickThresholdPx) {
        clearProvisional();
        return; // stay armed
    }
    // Long drag ended on empty space — cancel.
    cancel();
}

void WireItem::contextMenuEvent(QGraphicsSceneContextMenuEvent *event) {
    if (!m_state) {
        return;
    }
    QMenu menu;
    QAction *renameAct = menu.addAction(QStringLiteral("Rename..."));
    QAction *chosen = menu.exec(event->screenPos());
    if (chosen == renameAct) {
        // Driver key is the net identity; ask for alias.
        bool ok = false;
        QString current; // fetched signal name not exposed; leave blank
        QString text =
            QInputDialog::getText(nullptr, QStringLiteral("Rename Wire"),
                                  QStringLiteral("Alias for %1:").arg(m_source_key), QLineEdit::Normal, current, &ok);
        if (ok) {
            m_state->set_alias(m_source_key, text.trimmed());
        }
    }
    event->accept();
}

// --- CanvasView --------------------------------------------------------------

class CanvasView : public QGraphicsView {
  public:
    CanvasView(QGraphicsScene *scene, AppState *state, QWidget *parent = nullptr)
        : QGraphicsView(scene, parent), m_state(state) {
        setRenderHint(QPainter::Antialiasing);
        // Rubber-band select on empty-canvas drag; instance/pin items still
        // handle their own press/drag/release as before.
        setDragMode(QGraphicsView::RubberBandDrag);
        setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
        setResizeAnchor(QGraphicsView::AnchorViewCenter);
        setAcceptDrops(true);
        viewport()->setAcceptDrops(true);
        setFocusPolicy(Qt::StrongFocus);
    }

    void setWireTool(WireTool *wt) { m_wire_tool = wt; }

  protected:
    void keyPressEvent(QKeyEvent *event) override {
        if (event->key() == Qt::Key_Escape) {
            if (m_wire_tool)
                m_wire_tool->cancel();
            if (scene())
                scene()->clearSelection();
            event->accept();
            return;
        }
        if (event->key() == Qt::Key_Backspace || event->key() == Qt::Key_Delete) {
            // Collect selected wires and instances in one pass, then apply.
            QVector<QPair<QString, QString>> wires_to_clear;
            QVector<QString> instances_to_remove;
            if (auto *s = scene()) {
                for (QGraphicsItem *it : s->selectedItems()) {
                    if (auto *wire = dynamic_cast<WireItem *>(it)) {
                        const QString &tk = wire->targetKey();
                        int dot = tk.indexOf(QChar('.'));
                        if (dot > 0) {
                            wires_to_clear.append({tk.left(dot), tk.mid(dot + 1)});
                        }
                    } else if (auto *inst = dynamic_cast<InstanceItem *>(it)) {
                        instances_to_remove.append(inst->instanceName());
                    }
                }
            }
            bool did_something = false;
            for (const auto &p : wires_to_clear) {
                if (m_state->clear_port_map_entry(p.first, p.second)) {
                    did_something = true;
                }
            }
            for (const QString &name : instances_to_remove) {
                if (m_state->remove_instance(name)) {
                    did_something = true;
                }
            }
            // Fallback: use single-selection state if nothing was in scene selection.
            if (!did_something) {
                QString sel = m_state->selected_instance();
                if (!sel.isEmpty() && m_state->remove_instance(sel)) {
                    did_something = true;
                }
            }
            if (did_something) {
                event->accept();
                return;
            }
        }
        QGraphicsView::keyPressEvent(event);
    }

    void wheelEvent(QWheelEvent *event) override {
        if (event->modifiers() & Qt::ControlModifier) {
            // Proportional to scroll magnitude: kZoomStep per standard wheel
            // notch (120 angle-delta units). Trackpad sends smaller increments
            // so each gesture step zooms less. Avoids the old "every tick =
            // 15 % jump" sensitivity.
            double delta = event->angleDelta().y();
            if (delta == 0)
                delta = event->pixelDelta().y() * 4.0;
            if (delta == 0) {
                event->accept();
                return;
            }
            double factor = std::pow(kZoomStep, delta / 120.0);
            double new_scale = m_zoom * factor;
            if (new_scale < kZoomMin || new_scale > kZoomMax) {
                event->accept();
                return;
            }
            m_zoom = new_scale;
            scale(factor, factor);
            event->accept();
        } else {
            QGraphicsView::wheelEvent(event);
        }
    }

    void mousePressEvent(QMouseEvent *event) override {
        if (event->button() == Qt::MiddleButton) {
            m_panAnchor = event->position().toPoint();
            m_panning = true;
            setCursor(Qt::ClosedHandCursor);
            event->accept();
            return;
        }
        // Detect empty-canvas left click BEFORE delegating to base class —
        // base may consume it for rubber-band drag start.
        bool empty_click = event->button() == Qt::LeftButton && itemAt(event->pos()) == nullptr;
        QGraphicsView::mousePressEvent(event);
        if (empty_click) {
            // Deselect: hides mini editor and unhighlights any instance.
            m_state->set_selected_instance(QString());
        }
    }

    void mouseMoveEvent(QMouseEvent *event) override {
        if (m_panning) {
            QPoint delta = event->position().toPoint() - m_panAnchor;
            m_panAnchor = event->position().toPoint();
            horizontalScrollBar()->setValue(horizontalScrollBar()->value() - delta.x());
            verticalScrollBar()->setValue(verticalScrollBar()->value() - delta.y());
            event->accept();
            return;
        }
        QGraphicsView::mouseMoveEvent(event);
    }

    void mouseReleaseEvent(QMouseEvent *event) override {
        if (event->button() == Qt::MiddleButton && m_panning) {
            m_panning = false;
            unsetCursor();
            event->accept();
            return;
        }
        QGraphicsView::mouseReleaseEvent(event);
    }

    void dragEnterEvent(QDragEnterEvent *event) override {
        if (event->mimeData()->hasFormat(QString::fromLatin1(kModuleMimeType))) {
            event->acceptProposedAction();
        }
    }

    void dragMoveEvent(QDragMoveEvent *event) override {
        if (event->mimeData()->hasFormat(QString::fromLatin1(kModuleMimeType))) {
            event->acceptProposedAction();
        }
    }

    void dropEvent(QDropEvent *event) override {
        QByteArray data = event->mimeData()->data(QString::fromLatin1(kModuleMimeType));
        if (data.isEmpty()) {
            return;
        }
        QString module = QString::fromUtf8(data);
        QString inst_name = allocate_instance_name(m_state, module);
        if (!m_state->add_instance(inst_name, module)) {
            return;
        }
        QPointF scene_pos = mapToScene(event->position().toPoint());
        // Snap drop X to the nearest column center; assume default min width
        // (real width measured after layoutPins lands the actual instance).
        qreal centered = scene_pos.x();
        int col = static_cast<int>(std::round(centered / kColumnPitch));
        qreal snapped_x = col * kColumnPitch - kMinInstanceWidth / 2.0;
        m_state->set_instance_position(inst_name, snapped_x, scene_pos.y());
        event->acceptProposedAction();
    }

  private:
    AppState *m_state;
    QPoint m_panAnchor;
    bool m_panning = false;
    double m_zoom = 1.0;
    WireTool *m_wire_tool = nullptr;
};

// --- CanvasLayer: manages scene <-> AppState sync ----------------------------

class CanvasLayer {
  public:
    CanvasLayer(QGraphicsScene *scene, AppState *state) : m_scene(scene), m_state(state), m_wire_tool(state, scene) {}

    WireTool *wireTool() { return &m_wire_tool; }

    void rebuild() {
        // Cancel any in-flight wire FIRST — scene->clear() deletes the
        // provisional QGraphicsPathItem, so calling cancel() afterwards would
        // double-delete.
        m_wire_tool.cancel();
        m_scene->clear();
        m_items.clear();
        m_top_ports.clear();
        m_top_port_by_name.clear();
        m_wires.clear();
        m_junction_dots.clear();
        int count = m_state->instance_count();
        for (int i = 0; i < count; ++i) {
            QString name = m_state->instance_name(i);
            QString module = m_state->instance_module(i);
            double x = m_state->instance_pos_x(i);
            double y = m_state->instance_pos_y(i);
            auto *item = new InstanceItem(m_state, name, module);
            item->setPos(x, y);
            item->setWireTool(&m_wire_tool);
            item->setCanvasLayer(this);
            m_scene->addItem(item);
            m_items.insert(name, item);
        }
        rebuildTopPorts();
        rebuildWires();
        QString sel = m_state->selected_instance();
        if (!sel.isEmpty()) {
            highlight(sel);
        }
    }

    // Column index of an instance, derived from its body center X.
    int instanceColumn(InstanceItem *item) const {
        qreal cx = item->pos().x() + item->width() / 2.0;
        return static_cast<int>(std::round(cx / static_cast<qreal>(kColumnPitch)));
    }

    // Min/max column over current instances. Empty canvas defaults to (0, 0).
    std::pair<int, int> columnBounds() const {
        if (m_items.isEmpty())
            return {0, 0};
        int lo = INT_MAX, hi = INT_MIN;
        for (auto it = m_items.constBegin(); it != m_items.constEnd(); ++it) {
            int c = instanceColumn(it.value());
            lo = std::min(lo, c);
            hi = std::max(hi, c);
        }
        return {lo, hi};
    }

    void rebuildTopPorts() {
        for (auto *t : m_top_ports) {
            m_scene->removeItem(t);
            delete t;
        }
        m_top_ports.clear();
        m_top_port_by_name.clear();
        int n = m_state->top_port_count();
        std::vector<int> inputs, outputs;
        for (int i = 0; i < n; ++i) {
            int d = m_state->top_port_direction(i);
            if (d == 0)
                inputs.push_back(i);
            else
                outputs.push_back(i);
        }
        // Top ports always sit one column beyond the outermost instance column,
        // so modules never share a column with top-level connectors.
        auto [col_lo, col_hi] = columnBounds();
        qreal in_x = (col_lo - 1) * static_cast<qreal>(kColumnPitch);
        qreal out_x = (col_hi + 1) * static_cast<qreal>(kColumnPitch);
        int in_total = static_cast<int>(inputs.size()) * kTopPortSpacing;
        int out_total = static_cast<int>(outputs.size()) * kTopPortSpacing;
        int in_y = -in_total / 2;
        int out_y = -out_total / 2;
        for (int i : inputs) {
            QString nm = m_state->top_port_name(i);
            auto *tp = new TopPortItem(nm, 0, m_state->top_port_width(i), PinSide::Left);
            tp->setPos(in_x, in_y);
            tp->setWireTool(&m_wire_tool);
            m_scene->addItem(tp);
            m_top_ports.push_back(tp);
            m_top_port_by_name.insert(nm, tp);
            in_y += kTopPortSpacing;
        }
        for (int i : outputs) {
            QString nm = m_state->top_port_name(i);
            auto *tp = new TopPortItem(nm, 1, m_state->top_port_width(i), PinSide::Right);
            tp->setPos(out_x, out_y);
            tp->setWireTool(&m_wire_tool);
            m_scene->addItem(tp);
            m_top_ports.push_back(tp);
            m_top_port_by_name.insert(nm, tp);
            out_y += kTopPortSpacing;
        }
    }

    struct Endpoint {
        QString key;
        QPointF pt;
        bool exits_right = false;
        int col = 0;
    };

    static int columnForX(qreal x) {
        return static_cast<int>(std::round(x / static_cast<qreal>(kColumnPitch)));
    }

    // Resolve a wire endpoint key to scene position, exit side, and column.
    // Keys: "top:<name>" or "<inst>.<port>".
    bool resolveEndpoint(const QString &key, Endpoint &out) const {
        out.key = key;
        if (key.startsWith(QStringLiteral("top:"))) {
            QString nm = key.mid(4);
            int bracket = nm.indexOf(QChar('['));
            if (bracket >= 0)
                nm.truncate(bracket);
            auto *tp = m_top_port_by_name.value(nm, nullptr);
            if (!tp)
                return false;
            out.pt = tp->tipScenePos();
            // Top-port input (Left side) emits to the right into the canvas;
            // top-port output (Right side) receives from the left.
            out.exits_right = (tp->side() == PinSide::Left);
            out.col = columnForX(out.pt.x());
            return true;
        }
        int dot = key.indexOf(QChar('.'));
        if (dot < 0)
            return false;
        QString inst = key.left(dot);
        QString port = key.mid(dot + 1);
        // Strip any slice suffix ("dout[7:4]" -> "dout") so anchor lookup
        // matches the unsliced port name. Slices encode the same pin geometry.
        int bracket = port.indexOf(QChar('['));
        if (bracket >= 0)
            port.truncate(bracket);
        auto *item = m_items.value(inst, nullptr);
        if (!item)
            return false;
        out.pt = item->portAnchorScenePos(port);
        QRectF r = item->sceneBoundingRect();
        out.exits_right = out.pt.x() >= r.center().x();
        out.col = columnForX(r.center().x());
        return true;
    }

    // Gutter helpers shared by planNet. Gutter between cols c and c+1 has
    // index 2c+1; a pin exiting right at col c picks first gutter 2c+1, left
    // picks 2c-1. Gutter center X = (idx+1)/2 * kColumnPitch.
    static int gutterIndex(int col, bool exits_right) {
        return exits_right ? (2 * col + 1) : (2 * col - 1);
    }
    static qreal gutterCenterX(int idx) {
        // Gutter between cols c and c+1 has idx = 2c+1 and sits at
        // (c + 0.5) * pitch = idx / 2 * pitch. Previously this returned
        // ((idx+1)/2) * pitch which is the NEXT column's center — off by
        // half a column. Bounded-both routing masked the bug because the
        // safe-range midpoint overrode this value; one-sided gutters fell
        // through to natural_x and routed verticals through module bodies.
        return idx / 2.0 * static_cast<qreal>(kColumnPitch);
    }
    static qreal nextLane(int idx, QHash<int, int> &counter) {
        int n = counter.value(idx, 0);
        counter[idx] = n + 1;
        if (n == 0)
            return 0.0;
        int sign = (n % 2 == 1) ? 1 : -1;
        int mag = (n + 1) / 2;
        return sign * mag * static_cast<qreal>(kWireLaneStep);
    }
    static void enforceStub(qreal pin_x, bool exits_right, qreal &gx) {
        if (exits_right)
            gx = std::max(gx, pin_x + static_cast<qreal>(kWireStubMin));
        else
            gx = std::min(gx, pin_x - static_cast<qreal>(kWireStubMin));
    }

    // Per-gutter precomputed routing constraints used by planNet to position
    // lane-offset trunk verticals while still honoring kWireStubMin on every
    // attached pin. Populated by replanWires before any net is planned.
    struct GutterInfo {
        qreal safe_min = -1e18; // gx must be >= this so right-exit stubs are >= kWireStubMin
        qreal safe_max = 1e18;  // gx must be <= this so left-exit stubs are >= kWireStubMin
        int nets_using = 0;     // distinct nets that route through this gutter
    };

    // Resolve a gutter's lane-offset X for the next net to occupy it. Bounded
    // and balanced inside the safe range: when nets_using > 1 the step
    // collapses to fit, so up to N lanes always fit without violating stub
    // minimums. One-sided gutters (only right-exit or only left-exit pins)
    // are unbounded on the open side and use kWireLaneStep.
    qreal allocateLaneX(int idx, const QHash<int, GutterInfo> &gutter_info,
                        QHash<int, int> &gutter_counter) const {
        const auto it = gutter_info.find(idx);
        GutterInfo gi = (it != gutter_info.end()) ? *it : GutterInfo{};
        const qreal natural_x = gutterCenterX(idx);
        const bool bounded_left = gi.safe_min > -1e17;
        const bool bounded_right = gi.safe_max < 1e17;
        int n = gutter_counter.value(idx, 0);
        gutter_counter[idx] = n + 1;

        if (bounded_left && bounded_right) {
            // Both sides constrained — center between safe bounds and squeeze
            // the per-lane step so all using nets fit inside [safe_min, safe_max].
            const qreal w = gi.safe_max - gi.safe_min;
            const qreal center = (gi.safe_min + gi.safe_max) / 2.0;
            int lane_slots = (std::max)(1, gi.nets_using);
            const qreal step = (std::min)(static_cast<qreal>(kWireLaneStep), w / static_cast<qreal>(lane_slots));
            qreal off = 0.0;
            if (n > 0) {
                int sign = (n % 2 == 1) ? 1 : -1;
                int mag = (n + 1) / 2;
                off = sign * mag * step;
            }
            qreal gx = center + off;
            gx = std::max(gx, gi.safe_min);
            gx = std::min(gx, gi.safe_max);
            return gx;
        }
        if (bounded_right) {
            // Fan leftward away from the constrained side. Lane n sits
            // kWireStubMin past the pin (lane 0) then steps further out for
            // higher indices. Never anchored on the boundary, so distinct
            // nets never collapse onto a clamped X.
            return gi.safe_max - static_cast<qreal>(n) * static_cast<qreal>(kWireLaneStep);
        }
        if (bounded_left) {
            // Fan rightward away from the constrained side.
            return gi.safe_min + static_cast<qreal>(n) * static_cast<qreal>(kWireLaneStep);
        }
        // Unbounded (no pins yet on either side — shouldn't happen post-pass-1):
        // zigzag around the natural gutter center.
        qreal off = 0.0;
        if (n > 0) {
            int sign = (n % 2 == 1) ? 1 : -1;
            int mag = (n + 1) / 2;
            off = sign * mag * static_cast<qreal>(kWireLaneStep);
        }
        return natural_x + off;
    }

    // Plan all paths for one net (driver + N loads) as a single tree. Sets
    // waypoints on each WireItem so a wire's stored path renders the
    // driver-to-its-load sub-route through the shared trunk. Emits filled
    // junction dots at interior trunk taps (where ≥ 3 segments meet).
    //
    // The horizontal bridge for cross-gutter nets runs at the AVERAGE Y of
    // all involved pins so the trunk stays in the natural vertical band of
    // its endpoints — not high above all modules. Bridges from different
    // nets land at distinct Y values whenever their pin sets differ.
    void planNet(const QString &src_key, const Endpoint &driver, const QVector<Endpoint> &loads,
                 const QVector<WireItem *> &wires,
                 QHash<int, int> &gutter_counter,
                 const QHash<int, GutterInfo> &gutter_info) {
        const int g_d = gutterIndex(driver.col, driver.exits_right);
        QVector<int> g_l(loads.size());
        QSet<int> unique_g;
        unique_g.insert(g_d);
        for (int i = 0; i < loads.size(); ++i) {
            g_l[i] = gutterIndex(loads[i].col, loads[i].exits_right);
            unique_g.insert(g_l[i]);
        }
        const QColor color = WireItem::colorForNet(src_key);
        QVector<QPointF> dot_points;

        if (unique_g.size() == 1) {
            // Single-gutter trunk: vertical run at one gutter X, every pin
            // (driver + loads) stubs horizontally to it.
            qreal gx = allocateLaneX(g_d, gutter_info, gutter_counter);
            for (int i = 0; i < loads.size(); ++i) {
                QVector<QPointF> wp;
                wp << driver.pt << QPointF(gx, driver.pt.y()) << QPointF(gx, loads[i].pt.y()) << loads[i].pt;
                wires[i]->setWaypoints(wp);
            }
            if (loads.size() >= 2) {
                QVector<qreal> ys;
                ys.push_back(driver.pt.y());
                for (const auto &l : loads)
                    ys.push_back(l.pt.y());
                std::sort(ys.begin(), ys.end());
                for (int i = 1; i < ys.size() - 1; ++i)
                    dot_points << QPointF(gx, ys[i]);
            }
        } else {
            // Multi-gutter tree: horizontal bridge at hy connecting one
            // vertical per unique gutter. Bridge Y = average of all pin Ys,
            // so the trunk runs through the natural band of the net rather
            // than far above every module.
            qreal sum_y = driver.pt.y();
            int count = 1;
            for (const auto &l : loads) {
                sum_y += l.pt.y();
                ++count;
            }
            qreal hy = sum_y / static_cast<qreal>(count);
            QHash<int, qreal> gx_for_idx;
            // Sort unique gutters for deterministic allocation order.
            QVector<int> sorted_gutters(unique_g.begin(), unique_g.end());
            std::sort(sorted_gutters.begin(), sorted_gutters.end());
            for (int idx : sorted_gutters)
                gx_for_idx[idx] = allocateLaneX(idx, gutter_info, gutter_counter);
            qreal dgx = gx_for_idx[g_d];

            for (int i = 0; i < loads.size(); ++i) {
                qreal lgx = gx_for_idx[g_l[i]];
                QVector<QPointF> wp;
                wp << driver.pt << QPointF(dgx, driver.pt.y()) << QPointF(dgx, hy);
                if (lgx != dgx)
                    wp << QPointF(lgx, hy);
                wp << QPointF(lgx, loads[i].pt.y()) << loads[i].pt;
                wires[i]->setWaypoints(wp);
            }

            // Interior trunk taps along the highway. Each unique gutter X
            // attaches one vertical at hy; trunk continues left+right at
            // interior taps, giving a T-junction (3 cardinal directions).
            QVector<qreal> tap_xs;
            tap_xs.append(gx_for_idx.values());
            std::sort(tap_xs.begin(), tap_xs.end());
            tap_xs.erase(std::unique(tap_xs.begin(), tap_xs.end()), tap_xs.end());
            for (int i = 1; i < tap_xs.size() - 1; ++i)
                dot_points << QPointF(tap_xs[i], hy);

            // Per-gutter drop junctions: when multiple loads share a gutter
            // (or a load shares the driver's gutter), the drop column has
            // its own interior Y taps.
            QHash<int, QVector<qreal>> ys_at_gutter;
            ys_at_gutter[g_d].push_back(driver.pt.y());
            for (int i = 0; i < loads.size(); ++i)
                ys_at_gutter[g_l[i]].push_back(loads[i].pt.y());
            for (auto it = ys_at_gutter.constBegin(); it != ys_at_gutter.constEnd(); ++it) {
                const auto &ys = it.value();
                if (ys.size() < 2)
                    continue;
                QVector<qreal> sorted = ys;
                sorted.push_back(hy);
                std::sort(sorted.begin(), sorted.end());
                qreal gx = gx_for_idx[it.key()];
                for (int i = 1; i < sorted.size() - 1; ++i) {
                    qreal y = sorted[i];
                    if (std::abs(y - hy) < 0.5)
                        continue; // hy already handled as trunk tap
                    dot_points << QPointF(gx, y);
                }
            }
        }

        QSet<QPair<int, int>> dedupe;
        for (const auto &p : dot_points) {
            auto key = qMakePair(static_cast<int>(std::round(p.x())), static_cast<int>(std::round(p.y())));
            if (dedupe.contains(key))
                continue;
            dedupe.insert(key);
            auto *dot = new JunctionDotItem(p, color);
            m_scene->addItem(dot);
            m_junction_dots.push_back(dot);
        }
    }


    void clearJunctionDots() {
        for (auto *d : m_junction_dots) {
            m_scene->removeItem(d);
            delete d;
        }
        m_junction_dots.clear();
    }

    // Drop any "[...]" slice suffix off a wire key so different slice variants
    // of the same physical driver pin collapse into one tree.
    static QString baseKey(const QString &key) {
        int b = key.indexOf(QChar('['));
        return (b >= 0) ? key.left(b) : key;
    }

    // Replan every wire path by grouping wires under their shared source_key
    // (one tree per net). Pass 1 resolves every net's endpoints and
    // accumulates per-gutter stub constraints + occupancy. Pass 2 hands the
    // computed GutterInfo to planNet so lane offsets scale to actually fit
    // inside the safe range, instead of collapsing to safe_min/safe_max.
    void replanWires() {
        clearJunctionDots();

        // Group by base (slice-stripped) source key: u_a.dout[7:4] and
        // u_a.dout[3:0] share one trunk because they originate from the same
        // physical pin.
        QHash<QString, QVector<WireItem *>> by_src;
        for (auto *w : m_wires)
            by_src[baseKey(w->sourceKey())].push_back(w);

        QStringList src_keys = by_src.keys();
        std::sort(src_keys.begin(), src_keys.end());

        struct NetData {
            Endpoint driver;
            QVector<Endpoint> loads;
            QVector<WireItem *> wires;
        };
        QHash<QString, NetData> nets;
        for (const QString &src_key : src_keys) {
            Endpoint driver;
            if (!resolveEndpoint(src_key, driver))
                continue;
            NetData d;
            d.driver = driver;
            for (auto *w : by_src[src_key]) {
                Endpoint l;
                if (!resolveEndpoint(w->targetKey(), l))
                    continue;
                d.loads.push_back(l);
                d.wires.push_back(w);
            }
            if (!d.loads.isEmpty())
                nets[src_key] = std::move(d);
        }

        // Pass 1: per-gutter constraints across all nets.
        QHash<int, GutterInfo> gutter_info;
        for (const QString &src_key : src_keys) {
            auto it = nets.find(src_key);
            if (it == nets.end())
                continue;
            const NetData &d = it.value();
            QSet<int> net_gutters;
            auto tighten = [&](const Endpoint &e) {
                int idx = gutterIndex(e.col, e.exits_right);
                net_gutters.insert(idx);
                GutterInfo &gi = gutter_info[idx];
                if (e.exits_right)
                    gi.safe_min = std::max(gi.safe_min, e.pt.x() + static_cast<qreal>(kWireStubMin));
                else
                    gi.safe_max = std::min(gi.safe_max, e.pt.x() - static_cast<qreal>(kWireStubMin));
            };
            tighten(d.driver);
            for (const auto &l : d.loads)
                tighten(l);
            for (int g : net_gutters)
                gutter_info[g].nets_using++;
        }

        // Pass 2: plan each net with lane allocator.
        QHash<int, int> gutter_counter;
        for (const QString &src_key : src_keys) {
            auto it = nets.find(src_key);
            if (it == nets.end())
                continue;
            const NetData &d = it.value();
            planNet(src_key, d.driver, d.loads, d.wires, gutter_counter, gutter_info);
        }
    }

    void rebuildWires() {
        for (auto *w : m_wires) {
            m_scene->removeItem(w);
            delete w;
        }
        m_wires.clear();
        clearJunctionDots();

        int wc = m_state->wire_count();
        for (int i = 0; i < wc; ++i) {
            QString src_key = m_state->wire_source(i);
            QString dst_key = m_state->wire_target(i);
            auto *w = new WireItem(src_key, dst_key);
            w->setAppState(m_state);
            w->setWidth(m_state->wire_width(i));
            w->setRouteIndex(static_cast<int>(m_wires.size()));
            w->setZValue(1);
            m_scene->addItem(w);
            m_wires.push_back(w);
        }
        replanWires();
    }

    // Same tree replanner; called when an instance moves so lane allocation
    // and trunk geometry follow the new layout.
    void rerouteWiresFor(const QString &inst_name) {
        Q_UNUSED(inst_name);
        replanWires();
    }

    void onInstanceAdded(const QString &name) {
        int idx = find_instance_index(m_state, name);
        if (idx < 0) {
            return;
        }
        QString module = m_state->instance_module(idx);
        double x = m_state->instance_pos_x(idx);
        double y = m_state->instance_pos_y(idx);
        auto *item = new InstanceItem(m_state, name, module);
        item->setPos(x, y);
        item->setWireTool(&m_wire_tool);
        item->setCanvasLayer(this);
        m_scene->addItem(item);
        m_items.insert(name, item);
        rebuildTopPorts();
        rebuildWires();
    }

    void onInstanceRemoved(const QString &name) {
        auto it = m_items.find(name);
        if (it == m_items.end()) {
            return;
        }
        m_scene->removeItem(it.value());
        delete it.value();
        m_items.erase(it);
        rebuildTopPorts();
        rebuildWires();
    }

    void onInstanceMoved(const QString &name, double x, double y) {
        auto it = m_items.find(name);
        if (it == m_items.end()) {
            return;
        }
        if (it.value()->pos() != QPointF(x, y)) {
            it.value()->setPos(x, y);
        }
        // Column bounds may have changed; reflow top ports so the diagram
        // stays bracketed by them.
        rebuildTopPorts();
        rerouteWiresFor(name);
    }

    // Called from InstanceItem::itemChange while the user drags. Reflows top
    // ports only when the column bounds shift (cheap during drag) and always
    // reroutes wires.
    void onInstanceColumnChanged() {
        auto bounds = columnBounds();
        if (bounds != m_last_col_bounds) {
            m_last_col_bounds = bounds;
            rebuildTopPorts();
        }
        rerouteWiresFor(QString());
    }

    void onPortMapChanged() {
        // Generic-map override can shift port widths — relayout pins (badges,
        // tip positions) before rerouting wires so anchors are current.
        for (auto it = m_items.begin(); it != m_items.end(); ++it) {
            it.value()->relayoutPins();
        }
        rebuildWires();
    }

    void highlight(const QString &name) {
        // Repaint all so the selected one flips to highlighted and others clear.
        for (auto it = m_items.begin(); it != m_items.end(); ++it) {
            it.value()->update();
        }
        Q_UNUSED(name);
    }

    InstanceItem *itemFor(const QString &name) const { return m_items.value(name, nullptr); }

  private:
    QGraphicsScene *m_scene;
    AppState *m_state;
    QHash<QString, InstanceItem *> m_items;
    std::vector<TopPortItem *> m_top_ports;
    QHash<QString, TopPortItem *> m_top_port_by_name;
    std::vector<WireItem *> m_wires;
    std::vector<JunctionDotItem *> m_junction_dots;
    std::pair<int, int> m_last_col_bounds = {0, 0};
    WireTool m_wire_tool;
};

// --- InstanceItem::itemChange (live wire reroute during drag) ---------------

QVariant InstanceItem::itemChange(GraphicsItemChange change, const QVariant &value) {
    // Snap X to nearest column center as the user drags. Y stays continuous.
    // Position frame: instance origin sits at the top-left; we want the
    // body's center (origin + width/2) to land on a column center, so the
    // origin's X must be `col*kColumnPitch - width/2`.
    if (change == ItemPositionChange) {
        QPointF p = value.toPointF();
        qreal centered = p.x() + m_width / 2.0;
        int col = static_cast<int>(std::round(centered / kColumnPitch));
        qreal snapped_x = col * kColumnPitch - m_width / 2.0;
        return QPointF(snapped_x, p.y());
    }
    if (change == ItemPositionHasChanged && m_canvas_layer) {
        m_canvas_layer->onInstanceColumnChanged();
    }
    return QGraphicsRectItem::itemChange(change, value);
}

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

// ---------------------------------------------------------------------------
// Mini editor: text view of a selected instance's generic/port map.
// ---------------------------------------------------------------------------

// Render one instance's bindings as a VHDL component-instantiation buffer.
// Returns an empty string if `instance_name` is empty or unknown.
//
// VHDL form:
//   u_fifo : fifo_sync
//     generic map (
//       WIDTH => 16,
//     )
//     port map (
//       clk => clk_sys,
//     );
//
// SV form (project_language == 1):
//   fifo_sync #(
//     .WIDTH(16),
//   ) u_fifo (
//     .clk(clk_sys),
//   );
//
// Trailing comma on every entry — punctuation is uniform; parser strips them.
static QString build_instance_buffer(AppState *state, const QString &instance_name) {
    if (instance_name.isEmpty())
        return QString();
    int idx = find_instance_index(state, instance_name);
    if (idx < 0)
        return QString();
    QString module = state->instance_module(idx);
    bool sv = state->project_language() == 1;

    QString out;
    if (state->instance_is_dirty(idx)) {
        QString prefix = sv ? QStringLiteral("//") : QStringLiteral("--");
        out += QStringLiteral("%1 Source file changed. Review the bindings below;\n"
                              "%1 ports whose direction/type changed were dropped by re-parse.\n")
                   .arg(prefix);
    }

    int gc = state->module_generic_count(idx);
    int pc = state->instance_port_count(idx);

    if (sv) {
        // SV: `module #( .P(v), ) inst ( .port(net), );`
        if (gc > 0) {
            out += QStringLiteral("%1 #(\n").arg(module);
            int name_width = 0;
            for (int g = 0; g < gc; ++g) {
                int w = static_cast<int>(state->module_generic_name(idx, g).size());
                if (w > name_width)
                    name_width = w;
            }
            for (int g = 0; g < gc; ++g) {
                QString gname = state->module_generic_name(idx, g);
                QString current = state->generic_map_entry(instance_name, gname);
                QString value =
                    current.isEmpty() ? state->module_generic_default(idx, g) : current;
                out += QStringLiteral("  .%1(%2),\n").arg(gname.leftJustified(name_width), value);
            }
            out += QStringLiteral(") %1 (\n").arg(instance_name);
        } else {
            out += QStringLiteral("%1 %2 (\n").arg(module, instance_name);
        }
        int name_width = 0;
        for (int p = 0; p < pc; ++p) {
            int w = static_cast<int>(state->instance_port_name(idx, p).size());
            if (w > name_width)
                name_width = w;
        }
        for (int p = 0; p < pc; ++p) {
            QString pname = state->instance_port_name(idx, p);
            QString rhs = state->port_map_entry(instance_name, pname);
            if (rhs.isEmpty())
                rhs = QStringLiteral("open");
            QString lhs = pname;
            QString slice = state->consumer_slice(instance_name, pname);
            if (!slice.isEmpty())
                lhs += slice;
            out += QStringLiteral("  .%1(%2),\n").arg(lhs.leftJustified(name_width), rhs);
        }
        out += QStringLiteral(");\n");
        return out;
    }

    // VHDL form
    out += QStringLiteral("%1 : %2\n").arg(instance_name, module);
    if (gc > 0) {
        out += QStringLiteral("  generic map (\n");
        int name_width = 0;
        for (int g = 0; g < gc; ++g) {
            int w = static_cast<int>(state->module_generic_name(idx, g).size());
            if (w > name_width)
                name_width = w;
        }
        for (int g = 0; g < gc; ++g) {
            QString gname = state->module_generic_name(idx, g);
            QString current = state->generic_map_entry(instance_name, gname);
            QString value = current.isEmpty() ? state->module_generic_default(idx, g) : current;
            out += QStringLiteral("    %1 => %2,\n").arg(gname.leftJustified(name_width), value);
        }
        out += QStringLiteral("  )\n");
    }

    out += QStringLiteral("  port map (\n");
    int name_width = 0;
    for (int p = 0; p < pc; ++p) {
        int w = static_cast<int>(state->instance_port_name(idx, p).size());
        if (w > name_width)
            name_width = w;
    }
    for (int p = 0; p < pc; ++p) {
        QString pname = state->instance_port_name(idx, p);
        QString rhs = state->port_map_entry(instance_name, pname);
        if (rhs.isEmpty())
            rhs = QStringLiteral("open");
        QString lhs = pname;
        QString slice = state->consumer_slice(instance_name, pname);
        if (!slice.isEmpty())
            lhs += slice;
        out += QStringLiteral("    %1 => %2,\n").arg(lhs.leftJustified(name_width), rhs);
    }
    out += QStringLiteral("  );\n");
    return out;
}

// Parse one `<name> => <rhs>` entry. Returns false on malformed RHS.
// Accepted RHS forms:
//   <identifier>                 → bare top-port / alias reference
//   <instance>.<port>            → instance-port driver
//   <driver>[<i>] | <driver>[<h>:<l>]  → slice
//   open                         → unconnected (returned as empty rhs string)
static bool parse_editor_line(const QString &name, const QString &rhs, QString *out_clean_rhs, QString *err) {
    QString r = rhs.trimmed();
    while (r.endsWith(QChar(',')))
        r.chop(1);
    r = r.trimmed();
    if (r.isEmpty()) {
        *err = QStringLiteral("%1: empty RHS").arg(name);
        return false;
    }
    if (r.compare(QStringLiteral("open"), Qt::CaseInsensitive) == 0) {
        *out_clean_rhs = QString(); // empty = open
        return true;
    }
    // Very permissive: allow identifiers, dots, brackets, digits, colons.
    static const QRegularExpression legal(
        QStringLiteral("^[A-Za-z_][A-Za-z0-9_]*(\\.[A-Za-z_][A-Za-z0-9_]*)?(\\[[0-9]+(:[0-9]+)?\\])?$"));
    if (!legal.match(r).hasMatch()) {
        *err = QStringLiteral("%1: cannot parse RHS '%2'").arg(name, r);
        return false;
    }
    *out_clean_rhs = r;
    return true;
}

// Extract (lhs, rhs) from a binding line. Recognizes both
//   VHDL form: `<name> => <value>[,]`
//   SV form:   `.<name>(<value>)[,]`
// Returns false if the line is a comment, blank, a section header, `);`, or
// the SV header line `module_name #(` / `) inst (`.
static bool extract_binding(const QString &line, QString *lhs, QString *rhs) {
    QString s = line.trimmed();
    if (s.isEmpty())
        return false;
    if (s.startsWith(QStringLiteral("--")) || s.startsWith(QStringLiteral("//")))
        return false;
    if (s.startsWith(QStringLiteral("generic map")) || s.startsWith(QStringLiteral("port map")))
        return false;
    if (s == QStringLiteral(")") || s == QStringLiteral(");"))
        return false;
    // SV header lines like `module_name #(` or `) u_inst (` carry no binding.
    if (s.endsWith(QStringLiteral("#(")) || s.endsWith(QStringLiteral("(")))
        return false;

    // SV form `.name(value),`
    if (s.startsWith(QChar('.'))) {
        int open = s.indexOf(QChar('('));
        int close = s.lastIndexOf(QChar(')'));
        if (open < 2 || close <= open)
            return false;
        QString name = s.mid(1, open - 1).trimmed();
        QString val = s.mid(open + 1, close - open - 1).trimmed();
        if (name.isEmpty())
            return false;
        *lhs = name;
        *rhs = val;
        return true;
    }

    // VHDL form `name => value,` — reject `name : type` lines.
    if (s.contains(QChar(':')) && !s.contains(QStringLiteral("=>")))
        return false;
    int arrow = s.indexOf(QStringLiteral("=>"));
    if (arrow < 0)
        return false;
    *lhs = s.left(arrow).trimmed();
    *rhs = s.mid(arrow + 2).trimmed();
    return !lhs->isEmpty();
}

// Result of parsing an editor buffer. Errors carry the offending line
// number so the inline highlighter can underline the right block.
struct EditorParseResult {
    QList<QPair<QString, QString>> generic_commits; // (name, rhs)
    QList<QPair<QString, QString>> port_commits;    // (name, rhs_clean)
    QList<QPair<int, QString>> errors;              // (line_index_0based, message)
};

static EditorParseResult parse_editor_buffer(const QString &buffer) {
    EditorParseResult r;
    enum Section { None, Generics, Ports };
    Section section = None;
    // Match SV header lines without language hint:
    //   `<module> #(`           → starts Generics
    //   `) <inst> (`            → ends Generics, starts Ports
    //   `<module> <inst> (`     → no-params header, starts Ports
    static const QRegularExpression sv_params_open(QStringLiteral("^[A-Za-z_][\\w]*\\s+#\\($"));
    static const QRegularExpression sv_params_to_ports(
        QStringLiteral("^\\)\\s+[A-Za-z_][\\w]*\\s+\\($"));
    static const QRegularExpression sv_ports_only(
        QStringLiteral("^[A-Za-z_][\\w]*\\s+[A-Za-z_][\\w]*\\s+\\($"));

    const QStringList lines = buffer.split(QChar('\n'));
    for (int i = 0; i < lines.size(); ++i) {
        const QString &raw = lines[i];
        QString trimmed = raw.trimmed();
        // VHDL section markers
        if (trimmed.startsWith(QStringLiteral("generic map"))) {
            section = Generics;
            continue;
        }
        if (trimmed.startsWith(QStringLiteral("port map"))) {
            section = Ports;
            continue;
        }
        // SV section markers
        if (sv_params_open.match(trimmed).hasMatch()) {
            section = Generics;
            continue;
        }
        if (sv_params_to_ports.match(trimmed).hasMatch()) {
            section = Ports;
            continue;
        }
        if (section == None && sv_ports_only.match(trimmed).hasMatch()) {
            section = Ports;
            continue;
        }

        QString lhs, rhs;
        if (!extract_binding(raw, &lhs, &rhs))
            continue;
        if (section == Ports) {
            QString clean;
            QString err;
            if (!parse_editor_line(lhs, rhs, &clean, &err)) {
                r.errors.append({i, err});
                continue;
            }
            r.port_commits.append({lhs, clean});
        } else if (section == Generics) {
            QString v = rhs;
            while (v.endsWith(QChar(',')))
                v.chop(1);
            r.generic_commits.append({lhs, v.trimmed()});
        }
    }
    return r;
}

// Pull an optional `[h:l]` or `[i]` suffix off a port name. On success, *port
// is the bare port name and *has_slice tells the caller whether to call
// set_consumer_slice or clear_consumer_slice.
static bool split_consumer_slice(const QString &lhs_in, QString *port, int *high, int *low, bool *has_slice) {
    int br_open = lhs_in.indexOf(QChar('['));
    if (br_open < 0) {
        *port = lhs_in;
        *has_slice = false;
        return true;
    }
    int br_close = lhs_in.lastIndexOf(QChar(']'));
    if (br_close <= br_open)
        return false;
    *port = lhs_in.left(br_open).trimmed();
    QString slice = lhs_in.mid(br_open + 1, br_close - br_open - 1).trimmed();
    int colon = slice.indexOf(QChar(':'));
    bool ok = false;
    if (colon < 0) {
        int v = slice.toInt(&ok);
        if (!ok)
            return false;
        *high = v;
        *low = v;
    } else {
        *high = slice.left(colon).trimmed().toInt(&ok);
        if (!ok)
            return false;
        *low = slice.mid(colon + 1).trimmed().toInt(&ok);
        if (!ok)
            return false;
    }
    *has_slice = true;
    return true;
}

// Commit a buffer back to the model. Returns the list of error messages;
// empty on success. On any parse error the model is NOT mutated.
static QStringList commit_editor_buffer(AppState *state, const QString &instance_name, const QString &buffer) {
    QStringList errors;
    if (instance_name.isEmpty())
        return errors;
    int idx = find_instance_index(state, instance_name);
    if (idx < 0)
        return errors;

    EditorParseResult parsed = parse_editor_buffer(buffer);
    for (const auto &e : parsed.errors)
        errors << e.second;
    if (!errors.isEmpty())
        return errors;

    for (const auto &p : parsed.generic_commits) {
        state->set_generic_map_entry(instance_name, p.first, p.second);
    }
    for (const auto &p : parsed.port_commits) {
        QString port_name;
        int slice_high = 0, slice_low = 0;
        bool has_slice = false;
        if (!split_consumer_slice(p.first, &port_name, &slice_high, &slice_low, &has_slice)) {
            errors << QStringLiteral("could not parse port slice in '%1'").arg(p.first);
            continue;
        }
        state->set_port_map_entry(instance_name, port_name, p.second);
        if (has_slice) {
            state->set_consumer_slice(instance_name, port_name, slice_high, slice_low);
        } else {
            state->clear_consumer_slice(instance_name, port_name);
        }
    }
    if (state->instance_is_dirty(idx)) {
        state->clear_instance_dirty(instance_name);
    }
    return errors;
}

// Underlines the RHS of port-map lines that fail validation. The set of
// bad line indices is recomputed on every textChanged in the editor.
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

// All RHS driver candidates: top-port names + `<instance>.<port>` strings.
// Excludes the instance currently being edited so users can't wire an
// instance to its own outputs by accident in the same buffer.
static QStringList rhs_candidates(AppState *state, const QString &editing_inst) {
    QStringList out;
    int tc = state->top_port_count();
    for (int i = 0; i < tc; ++i) {
        out << state->top_port_name(i);
    }
    int ic = state->instance_count();
    for (int i = 0; i < ic; ++i) {
        QString iname = state->instance_name(i);
        if (iname == editing_inst)
            continue;
        int pc = state->instance_port_count(i);
        for (int p = 0; p < pc; ++p) {
            out << QStringLiteral("%1.%2").arg(iname, state->instance_port_name(i, p));
        }
    }
    out.sort();
    return out;
}

// Ports of one instance, used when the user types `<inst>.` and triggers
// dot-completion.
static QStringList instance_port_candidates(AppState *state, const QString &inst_name) {
    QStringList out;
    int ic = state->instance_count();
    for (int i = 0; i < ic; ++i) {
        if (state->instance_name(i) != inst_name)
            continue;
        int pc = state->instance_port_count(i);
        for (int p = 0; p < pc; ++p) {
            out << state->instance_port_name(i, p);
        }
        break;
    }
    out.sort();
    return out;
}

// Detect completer context at the cursor. Returns kind + prefix to filter by.
//   None    — cursor not in a completable spot; popup should hide.
//   Rhs     — anywhere in RHS of `=>` line; offer all drivers.
//   DotPort — right after `<inst>.`; offer that instance's ports only.
struct CompletionContext {
    enum Kind { None, Rhs, DotPort } kind = None;
    QString prefix;   // chars typed so far (popup filter)
    QString instance; // for DotPort: the instance name before the dot
};

static CompletionContext detect_completion_context(const QString &line_before_cursor) {
    CompletionContext ctx;
    int arrow = line_before_cursor.indexOf(QStringLiteral("=>"));
    if (arrow < 0)
        return ctx;

    // Slice off the RHS portion.
    QString rhs = line_before_cursor.mid(arrow + 2);

    // Trailing run of identifier chars (letters, digits, _, .).
    int n = rhs.length();
    int i = n;
    while (i > 0) {
        QChar c = rhs[i - 1];
        if (c.isLetterOrNumber() || c == QChar('_') || c == QChar('.')) {
            --i;
        } else {
            break;
        }
    }
    QString tail = rhs.mid(i);

    int dot = tail.indexOf(QChar('.'));
    if (dot >= 0) {
        ctx.kind = CompletionContext::DotPort;
        ctx.instance = tail.left(dot);
        ctx.prefix = tail.mid(dot + 1);
    } else {
        ctx.kind = CompletionContext::Rhs;
        ctx.prefix = tail;
    }
    return ctx;
}

static QString default_open_dir() {
    QSettings settings(QStringLiteral("hdl-compose"), QStringLiteral("hdl-compose"));
    QString dir = settings.value(QStringLiteral("default_open_dir")).toString().trimmed();
    if (dir.isEmpty()) {
        dir = QStandardPaths::writableLocation(QStandardPaths::DocumentsLocation);
    }
    return dir;
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

extern "C" int run_gui(int *argc, char **argv) {
    // HiDPI: pass-through fractional scale factors (e.g. 2x Retina) without
    // rounding — keeps fonts and pixmaps crisp on macOS.
    QGuiApplication::setHighDpiScaleFactorRoundingPolicy(Qt::HighDpiScaleFactorRoundingPolicy::PassThrough);

    QApplication app(*argc, argv);
    app.setOrganizationName(QStringLiteral("hdl-compose"));
    app.setApplicationName(QStringLiteral("HDL Compose"));
    app.setStyle(QStringLiteral("Fusion"));
    apply_material_dark_theme(app);

    QMainWindow window;
    {
        QScreen *screen = QGuiApplication::primaryScreen();
        const QRect avail = screen ? screen->availableGeometry() : QRect(0, 0, 1400, 900);
        const int w = std::max(1024, static_cast<int>(avail.width() * 0.85));
        const int h = std::max(720, static_cast<int>(avail.height() * 0.85));
        window.resize(w, h);
        window.move(avail.center() - QPoint(w / 2, h / 2));
    }

    auto *state = new AppState(&window);
    const QIcon dirty_icon = make_dirty_icon();

    auto *root_splitter = new QSplitter(Qt::Horizontal, &window);

    // --- Sidebar ---
    auto *sidebar_splitter = new QSplitter(Qt::Vertical, root_splitter);

    auto *tree_model = new QStandardItemModel(&window);
    auto *tree_view = new QTreeView(sidebar_splitter);
    tree_view->setModel(tree_model);
    tree_view->setHeaderHidden(true);
    tree_view->setMinimumWidth(200);
    tree_view->setContextMenuPolicy(Qt::CustomContextMenu);
    tree_view->setSelectionMode(QAbstractItemView::SingleSelection);

    auto *library_label = new QLabel(QStringLiteral("Library"));
    library_label->setContentsMargins(4, 4, 4, 0);
    auto *library_view = new LibraryView(sidebar_splitter);
    auto *library_model = new QStringListModel(&window);
    library_view->setModel(library_model);
    auto *library_container = new QWidget(sidebar_splitter);
    auto *lib_layout = new QVBoxLayout(library_container);
    lib_layout->setContentsMargins(0, 0, 0, 0);
    lib_layout->setSpacing(0);
    lib_layout->addWidget(library_label);
    lib_layout->addWidget(library_view);

    sidebar_splitter->addWidget(tree_view);
    sidebar_splitter->addWidget(library_container);
    sidebar_splitter->setSizes({500, 300});

    // --- Canvas ---
    auto *scene = new QGraphicsScene(root_splitter);
    scene->setSceneRect(-2000, -2000, 4000, 4000);
    auto *canvas = new CanvasView(scene, state, root_splitter);
    canvas->setMinimumWidth(600);
    CanvasLayer canvas_layer(scene, state);
    canvas->setWireTool(canvas_layer.wireTool());

    // --- Mini editor ---
    // Wrap editor in a panel with a toggle button row above it. Toggle flips
    // between per-instance editing (default) and top-level entity editing.
    auto *editor_panel = new QWidget(root_splitter);
    editor_panel->setMinimumWidth(300);
    auto *editor_layout = new QVBoxLayout(editor_panel);
    editor_layout->setContentsMargins(0, 0, 0, 0);
    editor_layout->setSpacing(2);
    auto *editor_top_level_btn = new QPushButton(QStringLiteral("Top Level"), editor_panel);
    editor_top_level_btn->setCheckable(true);
    editor_top_level_btn->setToolTip(
        QStringLiteral("Edit the top-level entity declaration: add/remove ports and generics."));
    editor_layout->addWidget(editor_top_level_btn);
    auto *editor = new QPlainTextEdit(editor_panel);
    // Stretch factor 1 so the editor absorbs all extra vertical space when
    // visible. The trailing stretch (factor 0) takes over when the editor is
    // hidden — without it, QVBoxLayout would center the lone button.
    editor_layout->addWidget(editor, 1);
    editor_layout->addStretch();
    editor->hide(); // shown only while an instance is selected or top-level mode active
    // Panel itself stays visible so the Top Level toggle button is always
    // reachable even when nothing is selected on the canvas.
    {
        QFont f(QStringLiteral("Menlo"));
        f.setStyleHint(QFont::Monospace);
        f.setFixedPitch(true);
        editor->setFont(f);
    }
    auto *top_level_mode = new bool(false);

    // Holder for the instance currently represented in the buffer. An
    // empty string means "no instance selected".
    auto *editor_inst = new QString();
    // True while the user is actively typing; suppresses auto-repopulate
    // from model-change signals to avoid clobbering an in-progress edit.
    auto *editor_editing = new bool(false);
    // True during programmatic setPlainText so our own write doesn't
    // flip the editing flag.
    auto *editor_suppressing = new bool(false);

    // Inline syntax highlighter — red squiggles on bad RHS lines, recomputed
    // on every text change. Replaces the modal-popup-on-commit behavior.
    auto *highlighter = new MiniEditorHighlighter(editor->document());

    // RHS / dot-completer. Popup driven manually since QPlainTextEdit
    // doesn't auto-attach to QCompleter the way QLineEdit does.
    auto *completer_model = new QStringListModel(editor);
    auto *completer = new QCompleter(completer_model, editor);
    completer->setWidget(editor);
    completer->setCompletionMode(QCompleter::PopupCompletion);
    completer->setCaseSensitivity(Qt::CaseInsensitive);

    // Tab on the popup accepts the currently-highlighted candidate. Default
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
    completer->popup()->installEventFilter(new TabAcceptFilter(completer, editor));

    // 300 ms idle debounce. textChanged restarts it; on timeout we run the
    // parser + highlighter + completer popup. Avoids flicker / cursor moves
    // while user is mid-keystroke.
    auto *parse_timer = new QTimer(editor);
    parse_timer->setSingleShot(true);
    parse_timer->setInterval(300);

    auto repopulate_editor = [=, &window]() {
        *editor_suppressing = true;
        if (*top_level_mode) {
            editor->setPlainText(state->top_level_buffer());
            editor->show();
        } else if (editor_inst->isEmpty()) {
            editor->clear();
            editor->hide();
        } else {
            editor->setPlainText(build_instance_buffer(state, *editor_inst));
            editor->show();
        }
        *editor_suppressing = false;
        *editor_editing = false;
        highlighter->setErrorLines({});
        completer->popup()->hide();
        parse_timer->stop();
    };

    auto commit_editor = [=, &window]() {
        if (!*editor_editing)
            return; // nothing to commit
        if (*top_level_mode) {
            if (!state->commit_top_level_buffer(editor->toPlainText())) {
                QString err = state->last_error();
                window.statusBar()->showMessage(
                    QStringLiteral("Top-level: %1").arg(err.isEmpty() ? QStringLiteral("commit refused") : err), 5000);
                return;
            }
            *editor_editing = false;
            window.statusBar()->showMessage(QStringLiteral("Top-level entity updated"), 2000);
            return;
        }
        if (editor_inst->isEmpty())
            return;
        QStringList errs = commit_editor_buffer(state, *editor_inst, editor->toPlainText());
        if (!errs.isEmpty()) {
            // Refuse silently: squiggles + status bar already told the user.
            // Editor stays as-is; user fixes and retries.
            window.statusBar()->showMessage(
                QStringLiteral("Mini editor: %1 parse error(s) — fix to commit").arg(errs.size()), 4000);
            return;
        }
        // Don't re-render: that would jump the cursor and clobber the user's
        // formatting. Just mark the buffer clean. Column normalization will
        // happen the next time selection_changed switches away.
        *editor_editing = false;
        window.statusBar()->showMessage(QStringLiteral("Mini editor changes applied"), 2000);
    };

    // Toggle: enter top-level mode → deselect any instance and load the
    // top-level entity buffer. Exit → repopulate from the still-selected
    // instance (or hide the editor if none).
    QObject::connect(editor_top_level_btn, &QPushButton::toggled, &window, [=](bool checked) {
        commit_editor(); // flush in-flight edit before swapping
        *top_level_mode = checked;
        if (checked) {
            state->set_selected_instance(QString());
            editor_inst->clear();
        }
        repopulate_editor();
    });

    // Lightweight: textChanged just restarts the debounce timer. All real
    // work (parsing, highlighter, completer popup) waits for 300 ms idle.
    QObject::connect(editor, &QPlainTextEdit::textChanged, &window, [=]() {
        if (*editor_suppressing)
            return;
        *editor_editing = true;
        parse_timer->start();
    });

    QObject::connect(parse_timer, &QTimer::timeout, &window, [=, &window]() {
        if (*top_level_mode) {
            // Top-level grammar isn't checked live; commit-time errors land
            // in the status bar instead of inline squiggles.
            highlighter->setErrorLines({});
            completer->popup()->hide();
            return;
        }
        EditorParseResult parsed = parse_editor_buffer(editor->toPlainText());
        QSet<int> err_lines;
        for (const auto &e : parsed.errors)
            err_lines.insert(e.first);
        highlighter->setErrorLines(err_lines);
        if (parsed.errors.isEmpty()) {
            window.statusBar()->clearMessage();
        } else {
            window.statusBar()->showMessage(QStringLiteral("Mini editor: %1 parse error(s)").arg(parsed.errors.size()));
        }

        // Completer popup based on cursor context.
        QTextCursor cur = editor->textCursor();
        QString line = cur.block().text();
        int pos_in_block = cur.positionInBlock();
        QString before = line.left(pos_in_block);
        CompletionContext ctx = detect_completion_context(before);

        if (ctx.kind == CompletionContext::None) {
            completer->popup()->hide();
            return;
        }

        QStringList items;
        if (ctx.kind == CompletionContext::DotPort) {
            items = instance_port_candidates(state, ctx.instance);
        } else {
            items = rhs_candidates(state, *editor_inst);
        }
        completer_model->setStringList(items);
        completer->setCompletionPrefix(ctx.prefix);
        if (completer->completionCount() == 0) {
            completer->popup()->hide();
            return;
        }
        completer->popup()->setCurrentIndex(completer->completionModel()->index(0, 0));
        QRect rect = editor->cursorRect();
        rect.setWidth(completer->popup()->sizeHintForColumn(0) +
                      completer->popup()->verticalScrollBar()->sizeHint().width());
        completer->complete(rect);
    });

    QObject::connect(completer, QOverload<const QString &>::of(&QCompleter::activated), &window,
                     [=](const QString &text) {
                         QTextCursor c = editor->textCursor();
                         int n = completer->completionPrefix().length();
                         if (n > 0) {
                             c.movePosition(QTextCursor::Left, QTextCursor::KeepAnchor, n);
                         }
                         c.insertText(text);
                         editor->setTextCursor(c);
                     });

    // Focus-out → commit. QPlainTextEdit has no direct focusOut signal;
    // install an event filter on the widget.
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
    editor->installEventFilter(new FocusOutFilter(commit_editor, editor));

    // Ctrl+Return also commits.
    auto *commit_sc = new QShortcut(QKeySequence(Qt::CTRL | Qt::Key_Return), editor);
    commit_sc->setContext(Qt::WidgetWithChildrenShortcut);
    QObject::connect(commit_sc, &QShortcut::activated, &window, commit_editor);

    root_splitter->addWidget(sidebar_splitter);
    root_splitter->addWidget(canvas);
    root_splitter->addWidget(editor_panel);
    root_splitter->setSizes({250, 800, 350});
    window.setCentralWidget(root_splitter);

    // --- Menu ---
    auto *fileMenu = window.menuBar()->addMenu(QStringLiteral("&File"));
    auto *newAct = fileMenu->addAction(QStringLiteral("&New..."));
    newAct->setShortcut(QKeySequence::New);
    auto *openAct = fileMenu->addAction(QStringLiteral("&Open..."));
    openAct->setShortcut(QKeySequence::Open);
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

    // Edit menu — operations on the selected instance.
    auto *editMenu = window.menuBar()->addMenu(QStringLiteral("&Edit"));
    auto *undoAct = editMenu->addAction(QStringLiteral("&Undo"));
    undoAct->setShortcut(QKeySequence::Undo);
    auto *redoAct = editMenu->addAction(QStringLiteral("&Redo"));
    redoAct->setShortcut(QKeySequence::Redo);
    editMenu->addSeparator();
    auto *matchByNameAct = editMenu->addAction(QStringLiteral("&Match Ports by Name"));
    matchByNameAct->setShortcut(QKeySequence(Qt::CTRL | Qt::Key_M));
    matchByNameAct->setToolTip(QStringLiteral("Connect unmapped ports on the selected instance to matching top-level "
                                              "ports (same name + direction + type)"));

    QObject::connect(undoAct, &QAction::triggered, &window, [state, &window]() {
        if (state->undo()) {
            window.statusBar()->showMessage(QStringLiteral("Undo"), 1500);
        }
    });
    QObject::connect(redoAct, &QAction::triggered, &window, [state, &window]() {
        if (state->redo()) {
            window.statusBar()->showMessage(QStringLiteral("Redo"), 1500);
        }
    });
    auto refresh_undo_actions = [state, undoAct, redoAct]() {
        undoAct->setEnabled(state->can_undo());
        redoAct->setEnabled(state->can_redo());
    };
    refresh_undo_actions();
    QObject::connect(state, &AppState::project_loaded, &window, refresh_undo_actions);
    QObject::connect(state, &AppState::port_map_changed, &window,
                     [refresh_undo_actions](const QString &, const QString &) { refresh_undo_actions(); });
    QObject::connect(state, &AppState::port_map_changed_bulk, &window, refresh_undo_actions);
    QObject::connect(state, &AppState::instance_added, &window,
                     [refresh_undo_actions](const QString &) { refresh_undo_actions(); });
    QObject::connect(state, &AppState::instance_removed, &window,
                     [refresh_undo_actions](const QString &) { refresh_undo_actions(); });

    // Toolbar — quick access to the most-used file actions. Shares QAction
    // pointers with the File menu so shortcuts, enable-state, and icons stay
    // in sync.
    auto *fileToolbar = window.addToolBar(QStringLiteral("File"));
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
    exitAct->setShortcut(QKeySequence::Quit);

    window.statusBar()->showMessage(QStringLiteral("Ready"));
    update_window_title(&window, state);

    auto refresh_sidebar = [tree_model, library_model, state, dirty_icon, tree_view]() {
        rebuild_tree_model(tree_model, state, dirty_icon);
        rebuild_library_model(library_model, state);
        tree_view->expandAll();
    };

    // Title + validation reactive
    QObject::connect(state, &AppState::project_nameChanged, &window,
                     [&window, state]() { update_window_title(&window, state); });
    QObject::connect(state, &AppState::dirtyChanged, &window,
                     [&window, state]() { update_window_title(&window, state); });
    QObject::connect(state, &AppState::validation_changed, &window, [&window, state]() {
        int errs = state->validation_error_count();
        int warns = state->validation_warning_count();
        window.statusBar()->showMessage(QStringLiteral("%1 error(s), %2 warning(s)").arg(errs).arg(warns));
    });

    // Sidebar + canvas reactive
    QObject::connect(state, &AppState::project_loaded, &window, [refresh_sidebar, &canvas_layer]() {
        refresh_sidebar();
        canvas_layer.rebuild();
    });
    QObject::connect(state, &AppState::instance_added, &window, [refresh_sidebar, &canvas_layer](const QString &name) {
        refresh_sidebar();
        canvas_layer.onInstanceAdded(name);
    });
    QObject::connect(state, &AppState::instance_removed, &window,
                     [refresh_sidebar, &canvas_layer](const QString &name) {
                         refresh_sidebar();
                         canvas_layer.onInstanceRemoved(name);
                     });
    QObject::connect(
        state, &AppState::instance_moved, &window,
        [&canvas_layer](const QString &name, double x, double y) { canvas_layer.onInstanceMoved(name, x, y); });
    QObject::connect(state, &AppState::port_map_changed, &window,
                     [&canvas_layer](const QString &, const QString &) { canvas_layer.onPortMapChanged(); });
    QObject::connect(state, &AppState::port_map_changed_bulk, &window,
                     [&canvas_layer]() { canvas_layer.onPortMapChanged(); });
    QObject::connect(state, &AppState::alias_changed, &window,
                     [&canvas_layer](const QString &) { canvas_layer.onPortMapChanged(); });
    QObject::connect(state, &AppState::library_changed, &window, refresh_sidebar);

    // Module re-parse: watch every library path and auto-reload when the
    // underlying file changes. The reload drops stale port_map entries and
    // flags affected instances dirty (red outline + sidebar red dot); the
    // user reviews and either reconnects or clears the dirty flag.
    auto *fs_watcher = new QFileSystemWatcher(&window);
    auto refresh_watcher = [fs_watcher, state]() {
        if (!fs_watcher->files().isEmpty()) {
            fs_watcher->removePaths(fs_watcher->files());
        }
        int n = state->library_path_count();
        QStringList existing;
        for (int i = 0; i < n; ++i) {
            QString p = state->library_path(i);
            if (QFileInfo::exists(p))
                existing << p;
        }
        if (!existing.isEmpty())
            fs_watcher->addPaths(existing);
    };
    QObject::connect(state, &AppState::library_changed, &window, refresh_watcher);
    QObject::connect(state, &AppState::project_loaded, &window, refresh_watcher);
    QObject::connect(fs_watcher, &QFileSystemWatcher::fileChanged, &window,
                     [&window, state, refresh_watcher](const QString &path) {
                         // Some editors rename-swap to save — re-add the path
                         // after a brief delay in case the watcher lost it.
                         QTimer::singleShot(50, &window, [state, refresh_watcher]() {
                             state->reload_library();
                             refresh_watcher();
                         });
                         window.statusBar()->showMessage(
                             QStringLiteral("Source changed: %1 — reloading").arg(QFileInfo(path).fileName()), 3000);
                     });
    QObject::connect(state, &AppState::selection_changed, &window,
                     [&canvas_layer, tree_view, tree_model, editor_inst, top_level_mode, editor_top_level_btn,
                      commit_editor, repopulate_editor](const QString &name) {
                         // Commit any outgoing edit against the previous instance
                         // before switching so the user doesn't lose work.
                         commit_editor();
                         // Selecting an instance kicks the editor out of
                         // top-level mode automatically.
                         if (!name.isEmpty() && *top_level_mode) {
                             *top_level_mode = false;
                             QSignalBlocker b(editor_top_level_btn);
                             editor_top_level_btn->setChecked(false);
                         }
                         canvas_layer.highlight(name);
                         *editor_inst = name;
                         repopulate_editor();
                         // Sync sidebar tree row
                         for (int row = 0; row < tree_model->rowCount(); ++row) {
                             auto *root_item = tree_model->item(row);
                             for (int c = 0; c < root_item->rowCount(); ++c) {
                                 auto *child = root_item->child(c);
                                 if (child->data(Qt::UserRole).toString() == name) {
                                     tree_view->setCurrentIndex(child->index());
                                     return;
                                 }
                             }
                         }
                     });

    // Model-change signals: refresh the mini editor only when it's not being
    // actively edited. Once the user is typing we wait for focus-out.
    auto editor_model_changed = [=]() {
        if (*editor_editing)
            return;
        repopulate_editor();
    };
    QObject::connect(state, &AppState::port_map_changed, &window,
                     [editor_model_changed](const QString &, const QString &) { editor_model_changed(); });
    QObject::connect(state, &AppState::port_map_changed_bulk, &window, editor_model_changed);
    QObject::connect(state, &AppState::project_loaded, &window, [editor_inst, repopulate_editor]() {
        editor_inst->clear();
        repopulate_editor();
    });

    // Tree: single-click → set selection via AppState
    QObject::connect(tree_view, &QTreeView::clicked, &window, [state](const QModelIndex &index) {
        QString name = index.data(Qt::UserRole).toString();
        if (name.isEmpty()) {
            return;
        }
        state->set_selected_instance(name);
    });

    // Tree: double-click → goto source
    QObject::connect(tree_view, &QTreeView::doubleClicked, &window, [&window, state](const QModelIndex &index) {
        QString inst_name = index.data(Qt::UserRole).toString();
        if (inst_name.isEmpty()) {
            return;
        }
        int idx = find_instance_index(state, inst_name);
        if (idx < 0) {
            return;
        }
        QString src = state->instance_source_path(idx);
        launch_goto_source(&window, src);
    });

    // Tree: right-click → context menu
    QObject::connect(tree_view, &QTreeView::customContextMenuRequested, &window,
                     [&window, state, tree_view](const QPoint &pos) {
                         QModelIndex idx = tree_view->indexAt(pos);
                         if (!idx.isValid()) {
                             return;
                         }
                         QString inst_name = idx.data(Qt::UserRole).toString();
                         if (inst_name.isEmpty()) {
                             return;
                         }
                         QMenu menu(tree_view);
                         QAction *renameAct = menu.addAction(QStringLiteral("Rename..."));
                         QAction *deleteAct = menu.addAction(QStringLiteral("Delete"));
                         QAction *chosen = menu.exec(tree_view->viewport()->mapToGlobal(pos));
                         if (chosen == renameAct) {
                             bool ok = false;
                             QString new_name =
                                 QInputDialog::getText(&window, QStringLiteral("Rename Instance"),
                                                       QStringLiteral("New name:"), QLineEdit::Normal, inst_name, &ok);
                             if (!ok || new_name.trimmed().isEmpty() || new_name == inst_name) {
                                 return;
                             }
                             if (!state->rename_instance(inst_name, new_name.trimmed())) {
                                 show_state_error(&window, state, QStringLiteral("Rename"));
                             }
                         } else if (chosen == deleteAct) {
                             auto btn = QMessageBox::question(&window, QStringLiteral("Delete Instance"),
                                                              QStringLiteral("Delete instance %1?").arg(inst_name));
                             if (btn != QMessageBox::Yes) {
                                 return;
                             }
                             if (!state->remove_instance(inst_name)) {
                                 show_state_error(&window, state, QStringLiteral("Delete"));
                             }
                         }
                     });

    // Menu actions
    QObject::connect(newAct, &QAction::triggered, &window, [&window, state]() {
        QString name;
        int lang = 0;
        if (!prompt_new_project(&window, name, lang)) {
            return;
        }
        if (!state->new_project(name, lang)) {
            show_state_error(&window, state, QStringLiteral("New Project"));
            return;
        }
        window.statusBar()->showMessage(QStringLiteral("Created new project: %1").arg(name), 3000);
    });

    QObject::connect(openAct, &QAction::triggered, &window, [&window, state]() {
        QString path = QFileDialog::getOpenFileName(&window, QStringLiteral("Open Project"), default_open_dir(),
                                                    QStringLiteral("HDL Compose Projects (*.hdlc)"));
        if (path.isEmpty()) {
            return;
        }
        if (!state->open_project(path)) {
            show_state_error(&window, state, QStringLiteral("Open Project"));
            return;
        }
        window.statusBar()->showMessage(QStringLiteral("Opened %1").arg(path), 3000);
    });

    QObject::connect(addSourceAct, &QAction::triggered, &window, [&window, state]() {
        if (!state->has_project()) {
            QMessageBox::information(&window, QStringLiteral("Add HDL Source"),
                                     QStringLiteral("Create or open a project first."));
            return;
        }
        QStringList paths =
            QFileDialog::getOpenFileNames(&window, QStringLiteral("Add HDL Source(s)"), default_open_dir(),
                                          QStringLiteral("HDL sources (*.vhd *.vhdl *.v *.sv);;All files (*)"));
        if (paths.isEmpty()) {
            return;
        }
        int added = 0;
        QStringList failed;
        for (const QString &p : paths) {
            if (state->add_library_path(p)) {
                added++;
            } else {
                QString err = state->last_error();
                failed << (err.isEmpty() ? p : QStringLiteral("%1 (%2)").arg(p, err));
            }
        }
        if (!failed.isEmpty()) {
            QMessageBox::warning(&window, QStringLiteral("Add HDL Source"),
                                 QStringLiteral("Failed to add:\n%1").arg(failed.join(QChar('\n'))));
        }
        window.statusBar()->showMessage(QStringLiteral("Added %1 source(s)").arg(added), 3000);
    });

    auto save_as = [&window, state]() -> bool {
        QString path = QFileDialog::getSaveFileName(&window, QStringLiteral("Save Project As"), default_open_dir(),
                                                    QStringLiteral("HDL Compose Projects (*.hdlc)"));
        if (path.isEmpty()) {
            return false;
        }
        if (!path.endsWith(QStringLiteral(".hdlc"), Qt::CaseInsensitive)) {
            path += QStringLiteral(".hdlc");
        }
        if (!state->save_project_as(path)) {
            show_state_error(&window, state, QStringLiteral("Save Project"));
            return false;
        }
        window.statusBar()->showMessage(QStringLiteral("Saved to %1").arg(path), 3000);
        return true;
    };

    QObject::connect(saveAct, &QAction::triggered, &window, [&window, state, save_as]() {
        if (!state->has_project()) {
            QMessageBox::information(&window, QStringLiteral("Save"), QStringLiteral("No project to save."));
            return;
        }
        if (state->save_project()) {
            window.statusBar()->showMessage(QStringLiteral("Saved"), 3000);
        } else {
            save_as();
        }
    });

    QObject::connect(saveAsAct, &QAction::triggered, &window, [save_as]() { save_as(); });

    QObject::connect(generateAct, &QAction::triggered, &window, [&window, state]() {
        if (!state->has_project()) {
            QMessageBox::information(&window, QStringLiteral("Generate HDL"), QStringLiteral("No project loaded."));
            return;
        }
        int lang = state->project_language();
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
            QMessageBox::warning(&window, QStringLiteral("Generate HDL"), QStringLiteral("Unknown project language."));
            return;
        }
        QString suggested = state->suggest_codegen_path();
        QString path =
            QFileDialog::getSaveFileName(&window, QStringLiteral("Generate %1").arg(lang_label), suggested, filter);
        if (path.isEmpty()) {
            return;
        }
        if (state->generate_code(path)) {
            window.statusBar()->showMessage(QStringLiteral("Generated %1").arg(path), 5000);
        } else {
            show_state_error(&window, state, QStringLiteral("Generate HDL"));
        }
    });

    QObject::connect(reloadAct, &QAction::triggered, &window, [&window, state]() {
        if (!state->has_project()) {
            QMessageBox::information(&window, QStringLiteral("Refresh Library"), QStringLiteral("No project loaded."));
            return;
        }
        if (state->reload_library()) {
            window.statusBar()->showMessage(QStringLiteral("Library refreshed"), 3000);
        } else {
            show_state_error(&window, state, QStringLiteral("Refresh Library"));
        }
    });

    QObject::connect(copySourcesAct, &QAction::triggered, &window, [&window, state]() {
        QString proj_path = state->current_project_path();
        if (proj_path.isEmpty()) {
            QMessageBox::information(&window, QStringLiteral("Copy Sources"),
                                     QStringLiteral("Save the project first so we know where to copy to."));
            return;
        }
        QDir proj_dir = QFileInfo(proj_path).absoluteDir();
        int n = state->library_path_count();
        if (n == 0) {
            QMessageBox::information(&window, QStringLiteral("Copy Sources"),
                                     QStringLiteral("No library sources to copy."));
            return;
        }
        auto btn = QMessageBox::question(
            &window, QStringLiteral("Copy Sources"),
            QStringLiteral("Copy %1 source file(s) into %2?").arg(n).arg(proj_dir.absolutePath()));
        if (btn != QMessageBox::Yes) {
            return;
        }
        int copied = 0;
        QStringList failures;
        QStringList originals;
        for (int i = 0; i < n; ++i) {
            originals << state->library_path(i);
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
                auto overwrite = QMessageBox::question(&window, QStringLiteral("Overwrite?"),
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
            state->remove_library_path(src);
            state->add_library_path(target);
            copied++;
        }
        QString msg = QStringLiteral("Copied %1 of %2 file(s).").arg(copied).arg(n);
        if (!failures.isEmpty()) {
            msg += QStringLiteral("\n\nIssues:\n%1").arg(failures.join(QChar('\n')));
        }
        QMessageBox::information(&window, QStringLiteral("Copy Sources"), msg);
    });

    QObject::connect(prefsAct, &QAction::triggered, &window, [&window]() { prompt_preferences(&window); });

    QObject::connect(exitAct, &QAction::triggered, &app, &QApplication::quit);

    QObject::connect(matchByNameAct, &QAction::triggered, &window, [&window, state]() {
        QString sel = state->selected_instance();
        if (sel.isEmpty()) {
            window.statusBar()->showMessage(QStringLiteral("Match by Name: select an instance first"), 3000);
            return;
        }
        int count = state->match_by_name(sel);
        if (count > 0) {
            window.statusBar()->showMessage(QStringLiteral("Matched %1 port(s) by name").arg(count), 3000);
        } else {
            window.statusBar()->showMessage(QStringLiteral("No matching top-level ports found for '%1'").arg(sel),
                                            3000);
        }
    });

    window.show();
    return app.exec();
}
