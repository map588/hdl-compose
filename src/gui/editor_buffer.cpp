// Implementation of the mini-editor buffer subsystem (see editor_buffer.h).

#include "editor_buffer.h"

#include "items.h" // find_instance_index

#include <QChar>
#include <QRegularExpression>
#include <QStringLiteral>

namespace hdlc {
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
QString build_instance_buffer(AppState *state, const QString &instance_name) {
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


EditorParseResult parse_editor_buffer(const QString &buffer) {
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
QStringList commit_editor_buffer(AppState *state, const QString &instance_name, const QString &buffer) {
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

    // Batch: one undo step, one validation pass, one bulk signal for the
    // whole buffer instead of one per binding line.
    const bool any = !parsed.generic_commits.isEmpty() || !parsed.port_commits.isEmpty();
    if (any)
        state->begin_batch();
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
    if (any)
        state->end_batch();
    if (state->instance_is_dirty(idx)) {
        state->clear_instance_dirty(instance_name);
    }
    return errors;
}


// All RHS driver candidates: top-port names + `<instance>.<port>` strings.
// Excludes the instance currently being edited so users can't wire an
// instance to its own outputs by accident in the same buffer.
QStringList rhs_candidates(AppState *state, const QString &editing_inst) {
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
QStringList instance_port_candidates(AppState *state, const QString &inst_name) {
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


CompletionContext detect_completion_context(const QString &line_before_cursor) {
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

} // namespace hdlc
