// Chip view: the device as nextpnr placed the design on it.
//
// The data is `<top>.floorplan.json`, which `hdlc build` writes (see
// `src/floorplan.rs`). Free bels are grey outlines. Each tile that holds
// cells of the design gets a colored square; the color is the cell kind
// that the tile holds most of. The used bels inside the tile are solid.
#pragma once

#include <QFile>
#include <QGraphicsEllipseItem>
#include <QGraphicsItem>
#include <QGraphicsLineItem>
#include <QGraphicsRectItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHash>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNativeGestureEvent>
#include <QPainter>
#include <QSet>
#include <QStringList>
#include <QStyleOptionGraphicsItem>
#include <QVector>
#include <QWheelEvent>
#include <QMouseEvent>
#include <algorithm>
#include <cmath>
#include <functional>

namespace chipview {

    // Order of `CellKind` in `src/floorplan.rs`.
    inline const QStringList& kindNames() {
        static const QStringList names {QStringLiteral("Logic"), QStringLiteral("Register"),
            QStringLiteral("Io"), QStringLiteral("Dsp"), QStringLiteral("Memory"),
            QStringLiteral("Clock"), QStringLiteral("Other")};
        return names;
    }

    inline QColor kindColor(int kind) {
        static const QColor colors[] = {QColor(0x3b, 0x82, 0xf6), QColor(0x06, 0xb6, 0xd4),
            QColor(0x22, 0xc5, 0x5e), QColor(0xa8, 0x55, 0xf7), QColor(0xf5, 0x9e, 0x0b),
            QColor(0xef, 0x44, 0x44), QColor(0x9c, 0xa3, 0xaf)};
        return colors[kind >= 0 && kind < 7 ? kind : 6];
    }

    inline const QColor kBackground(24, 24, 28);
    inline const QColor kFreeBel(92, 92, 100);
    inline const QColor kHighlight(250, 204, 21);

    // All bel outlines in one item. The device has about 100 000 bels, so the
    // item skips each bel that is smaller than two pixels at the current zoom.
    class BelLayer : public QGraphicsItem {
    public:
        BelLayer(QVector<QRectF> free, QVector<QRectF> used, QRectF bounds)
            : m_free(std::move(free))
            , m_used(std::move(used))
            , m_bounds(bounds) { }

        QRectF boundingRect() const override { return m_bounds; }

        void paint(QPainter* painter, const QStyleOptionGraphicsItem* option, QWidget*) override {
            const qreal lod
                = QStyleOptionGraphicsItem::levelOfDetailFromTransform(painter->worldTransform());
            const QRectF exposed = option->exposedRect;
            const auto visible = [&](const QRectF& r) {
                return std::max(r.width(), r.height()) * lod >= 2.0 && r.intersects(exposed);
            };
            painter->setPen(QPen(kFreeBel, 0));
            painter->setBrush(Qt::NoBrush);
            for (const QRectF& r : m_free) {
                if (visible(r))
                    painter->drawRect(r);
            }
            painter->setPen(QPen(Qt::white, 0));
            painter->setBrush(QColor(0x60, 0xa5, 0xfa));
            for (const QRectF& r : m_used) {
                if (visible(r))
                    painter->drawRect(r);
            }
        }

    private:
        QVector<QRectF> m_free;
        QVector<QRectF> m_used;
        QRectF m_bounds;
    };

    class FloorplanView : public QGraphicsView {
    public:
        explicit FloorplanView(QWidget* parent = nullptr)
            : QGraphicsView(parent) {
            setScene(new QGraphicsScene(this));
            setBackgroundBrush(kBackground);
            setRenderHint(QPainter::Antialiasing, false);
            setDragMode(QGraphicsView::ScrollHandDrag);
            setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
            setViewportUpdateMode(QGraphicsView::SmartViewportUpdate);
        }

        // Load a chip view file. Returns false and leaves the view empty when
        // the file is absent or is not a chip view.
        bool load(const QString& path) {
            scene()->clear(); // Also deletes the path items.
            m_path_items.clear();
            m_paths.clear();
            m_tiles.clear();
            m_summary.clear();
            QFile f(path);
            if (!f.open(QIODevice::ReadOnly))
                return false;
            const QJsonObject root = QJsonDocument::fromJson(f.readAll()).object();
            const qreal pitch = root.value(QStringLiteral("tile_pitch")).toDouble();
            if (pitch <= 0)
                return false;

            const auto rects = [&](const char* key) {
                const QJsonArray a = root.value(QLatin1String(key)).toArray();
                QVector<QRectF> out;
                out.reserve(a.size() / 4);
                for (int i = 0; i + 3 < a.size(); i += 4)
                    out.append(QRectF(a[i].toDouble(), a[i + 1].toDouble(), a[i + 2].toDouble(),
                        a[i + 3].toDouble()));
                return out;
            };
            const QRectF bounds(0, 0, root.value(QStringLiteral("width")).toDouble(),
                root.value(QStringLiteral("height")).toDouble());
            scene()->setSceneRect(bounds.adjusted(-pitch, -pitch, pitch, pitch));
            scene()->addItem(new BelLayer(rects("free"), rects("used"), bounds));

            struct Tile {
                int kinds[7] = {};
                QStringList lines;
                QHash<QString, int> owners;
            };
            QHash<QPair<int, int>, Tile> tiles;
            const QJsonArray cells = root.value(QStringLiteral("cells")).toArray();
            for (const QJsonValue& v : cells) {
                const QJsonObject c = v.toObject();
                Tile& t = tiles[{
                    c.value(QStringLiteral("x")).toInt(), c.value(QStringLiteral("y")).toInt()}];
                const int kind = kindNames().indexOf(c.value(QStringLiteral("kind")).toString());
                t.kinds[kind < 0 ? 6 : kind]++;
                t.lines << QStringLiteral("%1  %2  %3")
                               .arg(c.value(QStringLiteral("bel")).toString(),
                                   c.value(QStringLiteral("cell_type")).toString(),
                                   c.value(QStringLiteral("name")).toString());
                for (const QJsonValue& seg : c.value(QStringLiteral("path")).toArray())
                    t.owners[seg.toString()]++;
            }
            for (auto it = tiles.cbegin(); it != tiles.cend(); ++it) {
                const Tile& t = it.value();
                const int kind = int(std::max_element(t.kinds, t.kinds + 7) - t.kinds);
                auto* item = scene()->addRect(
                    QRectF(it.key().first * pitch, it.key().second * pitch, pitch, pitch));
                item->setZValue(-1); // Below the bel outlines.
                item->setData(kKindRole, kind);
                QStringList lines = t.lines.mid(0, kTooltipLines);
                if (t.lines.size() > kTooltipLines)
                    lines << QStringLiteral("... %1 more").arg(t.lines.size() - kTooltipLines);
                item->setToolTip(QStringLiteral("Tile X%1/Y%2: %3 cells\n")
                                     .arg(it.key().first)
                                     .arg(it.key().second)
                                     .arg(t.lines.size())
                    + lines.join(QChar('\n')));
                m_tiles.append({item, t.owners});
            }

            QStringList parts;
            const QJsonObject util = root.value(QStringLiteral("utilization")).toObject();
            for (auto it = util.constBegin(); it != util.constEnd(); ++it) {
                const QJsonObject u = it.value().toObject();
                const qint64 used = u.value(QStringLiteral("used")).toInteger();
                if (used > 0)
                    parts << QStringLiteral("%1 %2/%3")
                                 .arg(it.key())
                                 .arg(used)
                                 .arg(u.value(QStringLiteral("available")).toInteger());
            }
            const QJsonObject fmax = root.value(QStringLiteral("fmax")).toObject();
            for (auto it = fmax.constBegin(); it != fmax.constEnd(); ++it)
                parts << QStringLiteral("%1 %2 MHz")
                             .arg(it.key())
                             .arg(it.value().toDouble(), 0, 'f', 1);
            m_summary = parts.join(QStringLiteral("   "));

            const auto centre = [&](const QJsonValue& loc) {
                const QJsonArray a = loc.toArray();
                return QPointF((a[0].toInt() + 0.5) * pitch, (a[1].toInt() + 0.5) * pitch);
            };
            const QJsonArray paths = root.value(QStringLiteral("critical_paths")).toArray();
            for (const QJsonValue& pv : paths) {
                const QJsonObject p = pv.toObject();
                Path path;
                const double delay = p.value(QStringLiteral("delay_ns")).toDouble();
                path.label = QStringLiteral("%1 ns   %2 -> %3")
                                 .arg(delay, 0, 'f', 2)
                                 .arg(p.value(QStringLiteral("from")).toString(),
                                     p.value(QStringLiteral("to")).toString());
                const QJsonValue budget = p.value(QStringLiteral("budget_ns"));
                if (budget.isDouble())
                    path.label += QStringLiteral("   (period %1 ns, slack %2 ns)")
                                      .arg(budget.toDouble(), 0, 'f', 2)
                                      .arg(budget.toDouble() - delay, 0, 'f', 2);
                for (const QJsonValue& hv : p.value(QStringLiteral("hops")).toArray()) {
                    const QJsonObject h = hv.toObject();
                    path.hops.append({h.value(QStringLiteral("kind")).toString(),
                        h.value(QStringLiteral("delay_ns")).toDouble(),
                        h.value(QStringLiteral("from_cell")).toString(),
                        h.value(QStringLiteral("to_cell")).toString(),
                        centre(h.value(QStringLiteral("from"))), centre(h.value(QStringLiteral("to"))),
                        h.value(QStringLiteral("net")).toString(),
                        h.value(QStringLiteral("source")).toString()});
                }
                m_paths.append(path);
            }
            m_pitch = pitch;

            highlight(m_highlighted);
            showPath(m_paths.isEmpty() ? -1 : 0);
            fit();
            return true;
        }

        // Utilization and clock frequency, for the label above the view.
        QString summary() const { return m_summary; }

        // Mark the tiles that hold cells of one schematic instance and dim the
        // other tiles. An empty name shows each tile at full strength.
        void highlight(const QString& instance) {
            m_highlighted = instance;
            bool any = false;
            for (const TileItem& t : std::as_const(m_tiles))
                any = any || t.owners.contains(instance);
            for (const TileItem& t : std::as_const(m_tiles)) {
                const bool hit = any && t.owners.contains(instance);
                QColor fill = kindColor(t.item->data(kKindRole).toInt());
                fill.setAlpha(!any ? 130 : hit ? 210 : 40);
                t.item->setBrush(fill);
                QPen outline(kHighlight, 2);
                outline.setCosmetic(true); // Two pixels at each zoom.
                t.item->setPen(hit ? outline : QPen(Qt::NoPen));
            }
        }

        // One line for each critical path: delay, launch and capture event,
        // and the slack when one clock does both.
        QStringList pathLabels() const {
            QStringList out;
            for (const Path& p : m_paths)
                out << p.label;
            return out;
        }

        // One line for each hop of a path: delay, type, cells, HDL source.
        QStringList hopLabels(int path) const {
            QStringList out;
            if (path < 0 || path >= m_paths.size())
                return out;
            for (const Hop& h : m_paths[path].hops)
                out << hopLabel(h);
            return out;
        }

        // Draw one critical path on the device; -1 draws none. A hop between
        // two tiles is a line, a hop inside a cell is a ring. The color goes
        // from yellow to red with the share of the path delay.
        void showPath(int path) {
            qDeleteAll(m_path_items);
            m_path_items.clear();
            m_path = path < m_paths.size() ? path : -1;
            if (m_path < 0)
                return;
            const QVector<Hop>& hops = m_paths[m_path].hops;
            double worst = 0.001;
            for (const Hop& h : hops)
                worst = std::max(worst, h.delay);
            for (const Hop& h : hops) {
                const double t = h.delay / worst;
                const QColor color(int(250 - 11 * t), int(204 - 136 * t), int(21 + 47 * t));
                QPen pen(color, kPathWidth);
                pen.setCosmetic(true);
                QGraphicsItem* item = nullptr;
                if (h.a != h.b) {
                    item = scene()->addLine(QLineF(h.a, h.b), pen);
                } else {
                    const qreal r = m_pitch * 0.35;
                    item = scene()->addEllipse(QRectF(h.a.x() - r, h.a.y() - r, 2 * r, 2 * r), pen);
                }
                item->setZValue(2); // Above tiles and bels.
                item->setToolTip(hopLabel(h));
                m_path_items.append(item);
            }
        }

        // Make one hop of the shown path thick, bring it into view, and
        // report the owners of its cells through `onTilePicked`.
        void selectHop(int hop) {
            if (m_path < 0 || hop < 0 || hop >= m_path_items.size())
                return;
            for (int i = 0; i < m_path_items.size(); ++i) {
                QPen pen = itemPen(m_path_items[i]);
                pen.setWidthF(i == hop ? kPathWidth * 2.5 : kPathWidth);
                setItemPen(m_path_items[i], pen);
            }
            ensureVisible(m_path_items[hop], 40, 40);
            if (!onTilePicked)
                return;
            // The same owner rule as `PlacedCell.path` in `src/floorplan.rs`.
            // The destination cell comes first; the source cell follows for
            // a destination that has no owner.
            const Hop& h = m_paths[m_path].hops[hop];
            QStringList owners;
            for (const QString& cell : {h.to_cell, h.from_cell}) {
                if (cell.startsWith(QChar('$')))
                    continue;
                QStringList path = cell.split(QChar('.'));
                path.removeLast();
                owners += path;
            }
            onTilePicked(owners);
        }

        // Called when the user clicks a tile that holds cells. The argument
        // holds each hierarchy name that owns a cell of the tile; the name
        // that owns the most cells is first.
        std::function<void(const QStringList&)> onTilePicked;

        void fit() {
            if (scene()->items().isEmpty())
                return;
            fitInView(scene()->sceneRect(), Qt::KeepAspectRatio);
            m_auto_fit = true;
        }

    protected:
        void wheelEvent(QWheelEvent* event) override {
            if (event->modifiers() & Qt::ControlModifier) {
                double delta = event->angleDelta().y();
                if (delta == 0)
                    delta = event->pixelDelta().y() * 4.0;
                zoomBy(std::pow(1.15, delta / 120.0));
                event->accept();
            } else {
                QGraphicsView::wheelEvent(event);
            }
        }

        // Trackpad pinch, the same gesture as on the schematic canvas.
        bool viewportEvent(QEvent* event) override {
            if (event->type() == QEvent::NativeGesture) {
                auto* gesture = static_cast<QNativeGestureEvent*>(event);
                if (gesture->gestureType() == Qt::ZoomNativeGesture) {
                    zoomBy(1.0 + gesture->value());
                    event->accept();
                    return true;
                }
            }
            return QGraphicsView::viewportEvent(event);
        }

        void mouseDoubleClickEvent(QMouseEvent*) override { fit(); }

        void mousePressEvent(QMouseEvent* event) override {
            m_press = event->pos();
            QGraphicsView::mousePressEvent(event);
        }

        // A press and a release at one point is a click on a tile. A longer
        // move is a pan.
        void mouseReleaseEvent(QMouseEvent* event) override {
            QGraphicsView::mouseReleaseEvent(event);
            if (event->button() != Qt::LeftButton || !onTilePicked
                || (event->pos() - m_press).manhattanLength() > 3)
                return;
            const QPointF at = mapToScene(event->pos());
            for (const TileItem& t : std::as_const(m_tiles)) {
                if (!t.item->rect().contains(at))
                    continue;
                QStringList names = t.owners.keys();
                std::sort(names.begin(), names.end(), [&](const QString& a, const QString& b) {
                    const int ca = t.owners.value(a), cb = t.owners.value(b);
                    return ca != cb ? ca > cb : a < b;
                });
                onTilePicked(names);
                return;
            }
        }

        // Keep the whole device in view until the user zooms. The dock gets
        // its size after the first load.
        void resizeEvent(QResizeEvent* event) override {
            QGraphicsView::resizeEvent(event);
            if (m_auto_fit)
                fit();
        }

    private:
        static constexpr int kKindRole = 0;
        static constexpr int kTooltipLines = 24;

        static constexpr qreal kPathWidth = 2.5;

        struct Hop {
            QString kind;
            double delay;
            QString from_cell, to_cell;
            QPointF a, b; // Tile centres of the two cells.
            QString net, source;
        };

        struct Path {
            QString label;
            QVector<Hop> hops;
        };

        static QString hopLabel(const Hop& h) {
            QString text = QStringLiteral("%1 ns  %2  ").arg(h.delay, 6, 'f', 2).arg(h.kind, -8);
            text += h.net.isEmpty() ? h.to_cell
                                    : QStringLiteral("%1:  %2 -> %3").arg(h.net, h.from_cell, h.to_cell);
            if (!h.source.isEmpty())
                text += QStringLiteral("   [%1]").arg(h.source);
            return text;
        }

        // A path item is a line or an ellipse; the two have no common pen
        // accessor.
        static QPen itemPen(QGraphicsItem* item) {
            if (auto* line = qgraphicsitem_cast<QGraphicsLineItem*>(item))
                return line->pen();
            return static_cast<QGraphicsEllipseItem*>(item)->pen();
        }

        static void setItemPen(QGraphicsItem* item, const QPen& pen) {
            if (auto* line = qgraphicsitem_cast<QGraphicsLineItem*>(item))
                line->setPen(pen);
            else
                static_cast<QGraphicsEllipseItem*>(item)->setPen(pen);
        }

        struct TileItem {
            QGraphicsRectItem* item;
            QHash<QString, int> owners; // Hierarchy name to cell count.
        };

        void zoomBy(double factor) {
            const double next = transform().m11() * factor;
            // From the whole device in a small dock to one bel across the view.
            if (next > 0.02 && next < 400.0) {
                scale(factor, factor);
                m_auto_fit = false;
            }
        }

        QVector<TileItem> m_tiles;
        QString m_summary;
        QString m_highlighted;
        QPoint m_press;
        QVector<Path> m_paths;
        QList<QGraphicsItem*> m_path_items; // Items of the shown path.
        int m_path = -1;
        qreal m_pitch = 50;
        bool m_auto_fit = true;
    };

} // namespace chipview
