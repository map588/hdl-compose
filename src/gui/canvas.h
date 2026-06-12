// Canvas layer: WireTool (interactive wiring), CanvasView (zoom/pan/drops),
// CanvasLayer (scene <-> AppState sync + routing planner).
//
// Class declarations + inline methods. Heavier out-of-line method bodies for
// WireTool (compatibilityError / tryCommit / onPin*) live in canvas.cpp.

#pragma once

#include "canvas_constants.h"
#include "items.h"

#include <QAction>
#include <QByteArray>
#include <QCursor>
#include <QDragEnterEvent>
#include <QDragMoveEvent>
#include <QDropEvent>
#include <QGraphicsPathItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHash>
#include <QKeyEvent>
#include <QMimeData>
#include <QMouseEvent>
#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <QPoint>
#include <QPointF>
#include <QRectF>
#include <QScrollBar>
#include <QSet>
#include <QStringList>
#include <QStringLiteral>
#include <QToolTip>
#include <QVBoxLayout>
#include <QVector>
#include <QWheelEvent>
#include <algorithm>
#include <climits>
#include <cmath>
#include <utility>
#include <vector>

namespace hdlc {

// --- WireTool ---------------------------------------------------------------

class WireTool {
  public:
    WireTool(AppState *state, QGraphicsScene *scene) : m_state(state), m_scene(scene) {}

    void onPinPressed(PortPinItem *pin, const QPointF &scene_pos);
    void onPinDragMove(const QPointF &scene_pos);
    void onPinReleased(const QPointF &scene_pos);

    void cancel();
    void notifyPinDestroyed(PortPinItem *pin) {
        if (m_armed == pin)
            m_armed = nullptr;
    }
    PortPinItem *armed() const { return m_armed; }
    QGraphicsItem *provisionalItem() const { return m_provisional; }
    void updateProvisional(const QPointF &scene_pos) { onPinDragMove(scene_pos); }

  private:
    bool tryCommit(PortPinItem *src, PortPinItem *dst);
    void createProvisional(PortPinItem *from, const QPointF &scene_pos);
    void clearProvisional();

    AppState *m_state;
    QGraphicsScene *m_scene;
    PortPinItem *m_armed = nullptr;
    QGraphicsPathItem *m_provisional = nullptr;
    QPointF m_press_pos;
    bool m_sticky_after_commit = false;
};

// --- CanvasView -------------------------------------------------------------

class CanvasView : public QGraphicsView {
  public:
    CanvasView(QGraphicsScene *scene, AppState *state, QWidget *parent = nullptr)
        : QGraphicsView(scene, parent), m_state(state) {
        setRenderHint(QPainter::Antialiasing);
        setDragMode(QGraphicsView::RubberBandDrag);
        setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
        setResizeAnchor(QGraphicsView::AnchorViewCenter);
        setAcceptDrops(true);
        viewport()->setAcceptDrops(true);
        setFocusPolicy(Qt::StrongFocus);
    }

    void setWireTool(WireTool *wt) { m_wire_tool = wt; }
    void setCanvasLayer(CanvasLayer *layer) { m_canvas_layer = layer; }

    void zoomToFit() {
        if (!scene())
            return;
        QRectF r = scene()->itemsBoundingRect();
        if (r.isEmpty())
            return;
        fitInView(r.adjusted(-80, -80, 80, 80), Qt::KeepAspectRatio);
        qreal s = transform().m11();
        qreal clamped = std::clamp(s, kZoomMin, kZoomMax);
        if (clamped != s)
            scale(clamped / s, clamped / s);
        m_zoom = clamped;
    }

  protected:
    void keyPressEvent(QKeyEvent *event) override {
        if (event->key() == Qt::Key_Escape) {
            if (m_wire_tool)
                m_wire_tool->cancel();
            if (scene())
                scene()->clearSelection();
            // Keep AppState in sync — otherwise the editor panel keeps
            // showing the deselected instance.
            m_state->set_selected_instance(QString());
            event->accept();
            return;
        }
        if ((event->key() == Qt::Key_F && event->modifiers() == Qt::NoModifier) ||
            (event->key() == Qt::Key_0 && (event->modifiers() & Qt::ControlModifier))) {
            zoomToFit();
            event->accept();
            return;
        }
        if (event->key() == Qt::Key_Backspace || event->key() == Qt::Key_Delete) {
            QVector<QPair<QString, QString>> wires_to_clear;
            QVector<QString> instances_to_remove;
            if (auto *s = scene()) {
                for (QGraphicsItem *it : s->selectedItems()) {
                    if (auto *wire = dynamic_cast<WireItem *>(it)) {
                        NetKey k = NetKey::parse(wire->targetKey());
                        if (k.valid && !k.is_top) {
                            wires_to_clear.append({k.instance, k.port});
                        }
                    } else if (auto *inst = dynamic_cast<InstanceItem *>(it)) {
                        instances_to_remove.append(inst->instanceName());
                    }
                }
            }
            bool did_something = false;
            // Batch multi-wire deletes: one undo step + one rebuild.
            const bool batch = wires_to_clear.size() > 1;
            if (batch)
                m_state->begin_batch();
            for (const auto &p : wires_to_clear) {
                if (m_state->clear_port_map_entry(p.first, p.second)) {
                    did_something = true;
                }
            }
            if (batch)
                m_state->end_batch();
            for (const QString &name : instances_to_remove) {
                if (m_state->remove_instance(name)) {
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
            double delta = event->angleDelta().y();
            if (delta == 0)
                delta = event->pixelDelta().y() * 4.0;
            if (delta == 0) {
                event->accept();
                return;
            }
            double factor = std::pow(kZoomStep, delta / 120.0);
            double new_scale = std::clamp(m_zoom * factor, kZoomMin, kZoomMax);
            if (new_scale == m_zoom) {
                event->accept();
                return;
            }
            scale(new_scale / m_zoom, new_scale / m_zoom);
            m_zoom = new_scale;
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
        QGraphicsItem *hit = itemAt(event->pos());
        // The provisional wire follows the cursor in click-click wiring mode;
        // it must not count as a "real" item under the click.
        if (m_wire_tool && hit == m_wire_tool->provisionalItem())
            hit = nullptr;
        bool empty_click = event->button() == Qt::LeftButton && hit == nullptr;
        QGraphicsView::mousePressEvent(event);
        if (empty_click) {
            if (m_wire_tool && m_wire_tool->armed())
                m_wire_tool->cancel();
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
        // Click-click wiring: keep the provisional line tracking the cursor
        // between the arming click and the closing click.
        if (m_wire_tool && m_wire_tool->armed())
            m_wire_tool->updateProvisional(mapToScene(event->position().toPoint()));
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

    void dropEvent(QDropEvent *event) override;

    void drawBackground(QPainter *painter, const QRectF &rect) override {
        QGraphicsView::drawBackground(painter, rect);
        // Faint guides at column centers so the snap target is visible.
        QPen pen(QColor(255, 255, 255, 10));
        pen.setCosmetic(true);
        painter->setPen(pen);
        int c0 = static_cast<int>(std::floor(rect.left() / kColumnPitch));
        int c1 = static_cast<int>(std::ceil(rect.right() / kColumnPitch));
        for (int c = c0; c <= c1; ++c) {
            qreal x = c * static_cast<qreal>(kColumnPitch);
            painter->drawLine(QPointF(x, rect.top()), QPointF(x, rect.bottom()));
        }
    }

  private:
    AppState *m_state;
    QPoint m_panAnchor;
    bool m_panning = false;
    double m_zoom = 1.0;
    WireTool *m_wire_tool = nullptr;
    CanvasLayer *m_canvas_layer = nullptr;
};

// --- CanvasLayer ------------------------------------------------------------

class CanvasLayer {
  public:
    CanvasLayer(QGraphicsScene *scene, AppState *state) : m_scene(scene), m_state(state), m_wire_tool(state, scene) {}

    WireTool *wireTool() { return &m_wire_tool; }

    void rebuild() {
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
            // Wire tool + canvas layer before setPos so the snap/clear-Y
            // logic in itemChange applies to positions loaded from disk —
            // legacy projects may have saved overlapping modules.
            item->setWireTool(&m_wire_tool);
            item->setCanvasLayer(this);
            m_scene->addItem(item);
            m_items.insert(name, item);
            item->setPos(x, y);
        }
        rebuildTopPorts();
        rebuildWires();
        QString sel = m_state->selected_instance();
        if (!sel.isEmpty()) {
            refreshSelectionHighlight();
        }
    }

    int instanceColumn(InstanceItem *item) const {
        qreal cx = item->pos().x() + item->width() / 2.0;
        return static_cast<int>(std::round(cx / static_cast<qreal>(kColumnPitch)));
    }

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

    bool resolveEndpoint(const QString &key, Endpoint &out) const {
        out.key = key;
        NetKey k = NetKey::parse(key);
        if (!k.valid)
            return false;
        if (k.is_top) {
            auto *tp = m_top_port_by_name.value(k.port, nullptr);
            if (!tp)
                return false;
            out.pt = tp->tipScenePos();
            out.exits_right = (tp->side() == PinSide::Left);
            out.col = columnForX(out.pt.x());
            return true;
        }
        auto *item = m_items.value(k.instance, nullptr);
        if (!item)
            return false;
        out.pt = item->portAnchorScenePos(k.port);
        QRectF r = item->sceneBoundingRect();
        out.exits_right = out.pt.x() >= r.center().x();
        out.col = columnForX(r.center().x());
        return true;
    }

    // Routing geometry (lane allocation, bridge planning, junction dots)
    // lives in Rust — src/routing.rs — reached via plan_routes_ffi /
    // resolve_clear_y_ffi below. The canvas only gathers pin positions and
    // module rects and applies the returned polylines.

    qreal resolveClearY(InstanceItem *self, qreal snapped_x, qreal w, qreal h, qreal proposed_y) const {
        rust::Vec<FfiRect> obstacles;
        for (auto it = m_items.constBegin(); it != m_items.constEnd(); ++it) {
            if (it.value() == self)
                continue;
            QRectF r = it.value()->sceneBoundingRect();
            obstacles.push_back(FfiRect{r.left(), r.top(), r.right(), r.bottom()});
        }
        return resolve_clear_y_ffi(snapped_x, w, h, proposed_y,
                                   static_cast<double>(kMinModuleVerticalGap) / 2.0,
                                   std::move(obstacles));
    }

    // Single authority for instance placement: snap X to the column grid,
    // shove Y clear of other modules. Interactive drags (itemChange) and
    // programmatic placement (drop) both go through here; `proposed` is the
    // item's top-left corner.
    QPointF placeInstance(InstanceItem *item, const QPointF &proposed) const {
        const qreal w = item->width();
        const qreal centered = proposed.x() + w / 2.0;
        const int col = static_cast<int>(std::round(centered / static_cast<qreal>(kColumnPitch)));
        const qreal snapped_x = col * kColumnPitch - w / 2.0;
        const qreal y = resolveClearY(item, snapped_x, w, item->rect().height(), proposed.y());
        return QPointF(snapped_x, y);
    }

    void clearJunctionDots() {
        for (auto *d : m_junction_dots) {
            m_scene->removeItem(d);
            delete d;
        }
        m_junction_dots.clear();
    }

    static QString baseKey(const QString &key) { return NetKey::base(key); }

    // Net hover-highlight: light up every wire of the hovered net so fan-out
    // is traceable. Empty key clears.
    void setHoveredNet(const QString &base) {
        if (m_hovered_net == base)
            return;
        m_hovered_net = base;
        for (auto *w : m_wires)
            w->setNetHover(!base.isEmpty() && baseKey(w->sourceKey()) == base);
    }

    // Keep the scene rect covering everything placed so far (plus working
    // margin) — the old fixed ±2000 rect made wide designs unreachable by
    // scrolling. Never shrinks below the default workspace.
    void updateSceneRect() {
        QRectF base(-2000, -2000, 4000, 4000);
        QRectF items = m_scene->itemsBoundingRect();
        if (!items.isNull())
            base = base.united(items.adjusted(-600, -600, 600, 600));
        if (m_scene->sceneRect() != base)
            m_scene->setSceneRect(base);
    }

    void replanWires() {
        clearJunctionDots();

        QHash<QString, QVector<WireItem *>> by_src;
        for (auto *w : m_wires)
            by_src[baseKey(w->sourceKey())].push_back(w);

        QStringList src_keys = by_src.keys();
        std::sort(src_keys.begin(), src_keys.end());

        // Gather net endpoints in stable (sorted-key) order — lane
        // assignment in the router depends on allocation order.
        rust::Vec<FfiNet> nets;
        QVector<QVector<WireItem *>> net_wires; // parallel to nets
        QVector<QColor> net_colors;             // parallel to nets
        for (const QString &src_key : src_keys) {
            Endpoint driver;
            if (!resolveEndpoint(src_key, driver))
                continue;
            FfiNet net;
            net.driver = FfiEndpoint{driver.pt.x(), driver.pt.y(), driver.exits_right, driver.col};
            QVector<WireItem *> wires;
            for (auto *w : by_src[src_key]) {
                Endpoint l;
                if (!resolveEndpoint(w->targetKey(), l))
                    continue;
                net.loads.push_back(FfiEndpoint{l.pt.x(), l.pt.y(), l.exits_right, l.col});
                wires.push_back(w);
            }
            if (wires.isEmpty())
                continue;
            nets.push_back(std::move(net));
            net_wires.push_back(wires);
            net_colors.push_back(WireItem::colorForNet(src_key));
        }

        rust::Vec<FfiRect> obstacles;
        for (auto it = m_items.constBegin(); it != m_items.constEnd(); ++it) {
            QRectF r = it.value()->sceneBoundingRect();
            obstacles.push_back(FfiRect{r.left(), r.top(), r.right(), r.bottom()});
        }

        FfiRouteResult routes = plan_routes_ffi(
            std::move(nets), std::move(obstacles),
            FfiRouteParams{static_cast<double>(kColumnPitch), static_cast<double>(kWireLaneStep),
                           static_cast<double>(kWireStubMin)});

        size_t wi = 0;
        for (const auto &wires : net_wires) {
            for (auto *w : wires) {
                const FfiWire &fw = routes.wires[wi++];
                QVector<QPointF> pts;
                pts.reserve(static_cast<int>(fw.points.size()));
                for (const FfiPoint &fp : fw.points)
                    pts << QPointF(fp.x, fp.y);
                w->setWaypoints(pts);
            }
        }
        for (const FfiDot &d : routes.dots) {
            auto *dot = new JunctionDotItem(QPointF(d.x, d.y), net_colors[d.net]);
            m_scene->addItem(dot);
            m_junction_dots.push_back(dot);
        }
        updateSceneRect();
    }

    void rebuildWires() {
        for (auto *w : m_wires) {
            m_scene->removeItem(w);
            delete w;
        }
        m_wires.clear();
        clearJunctionDots();
        m_hovered_net.clear();

        int wc = m_state->wire_count();
        for (int i = 0; i < wc; ++i) {
            QString src_key = m_state->wire_source(i);
            QString dst_key = m_state->wire_target(i);
            auto *w = new WireItem(src_key, dst_key);
            w->setAppState(m_state);
            w->setCanvasLayer(this);
            w->setWidth(m_state->wire_width(i));
            w->setZValue(1);
            m_scene->addItem(w);
            m_wires.push_back(w);
        }
        replanWires();
    }

    void onInstanceAdded(const QString &name) {
        int idx = find_instance_index(m_state, name);
        if (idx < 0)
            return;
        QString module = m_state->instance_module(idx);
        double x = m_state->instance_pos_x(idx);
        double y = m_state->instance_pos_y(idx);
        auto *item = new InstanceItem(m_state, name, module);
        item->setWireTool(&m_wire_tool);
        item->setCanvasLayer(this);
        m_scene->addItem(item);
        m_items.insert(name, item);
        item->setPos(x, y);
        rebuildTopPorts();
        rebuildWires();
    }

    void onInstanceRemoved(const QString &name) {
        auto it = m_items.find(name);
        if (it == m_items.end())
            return;
        m_scene->removeItem(it.value());
        delete it.value();
        m_items.erase(it);
        rebuildTopPorts();
        rebuildWires();
    }

    void onInstanceMoved(const QString &name, double x, double y) {
        auto it = m_items.find(name);
        if (it == m_items.end())
            return;
        if (it.value()->pos() != QPointF(x, y)) {
            it.value()->setPos(x, y);
        }
        rebuildTopPorts();
        replanWires();
    }

    void onInstanceColumnChanged() {
        auto bounds = columnBounds();
        if (bounds != m_last_col_bounds) {
            m_last_col_bounds = bounds;
            rebuildTopPorts();
        }
        replanWires();
    }

    // Per-entry change: only the named instance's pins can be affected
    // (pin layout depends on ports/bundles/generics, not on other
    // instances' port maps). Wires always rebuild — routing is global.
    void onPortMapChanged(const QString &inst_name) {
        if (auto *item = m_items.value(inst_name, nullptr))
            item->relayoutPins();
        rebuildWires();
    }

    // Bulk change (generic override, editor commit, undo of a batch):
    // anything may have moved, so relayout everything.
    void onPortMapChangedBulk() {
        for (auto it = m_items.begin(); it != m_items.end(); ++it) {
            it.value()->relayoutPins();
        }
        rebuildWires();
    }

    // Repaint every instance so selection borders track AppState's
    // selected_instance (InstanceItem::paint reads it directly).
    void refreshSelectionHighlight() {
        for (auto it = m_items.begin(); it != m_items.end(); ++it) {
            it.value()->update();
        }
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
    QString m_hovered_net;
    WireTool m_wire_tool;
};

} // namespace hdlc
