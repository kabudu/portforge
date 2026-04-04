// PortForge Web Dashboard — Client-side JavaScript

// ─── Port Detail Modal ───

async function showDetail(port) {
    try {
        const response = await fetch(`/api/ports/${port}`);
        if (!response.ok) throw new Error('Port not found');
        const entry = await response.json();

        const modal = document.getElementById('detail-modal');
        const body = document.getElementById('modal-body');

        body.innerHTML = renderDetail(entry);
        modal.style.display = 'flex';

        // Close on Escape
        document.addEventListener('keydown', function handler(e) {
            if (e.key === 'Escape') {
                closeModal();
                document.removeEventListener('keydown', handler);
            }
        });
    } catch (err) {
        console.error('Failed to load port detail:', err);
    }
}

function closeModal() {
    document.getElementById('detail-modal').style.display = 'none';
}

function renderDetail(entry) {
    let html = `
        <h2 style="margin-bottom: 20px; display: flex; align-items: center; gap: 8px;">
            <span style="color: var(--accent-purple); font-family: 'JetBrains Mono', monospace;">:${entry.port}</span>
            <span class="status-badge ${getStatusClass(entry.status)}">${getStatusText(entry.status)}</span>
        </h2>

        <div class="detail-section">
            <h3>🔌 Process</h3>
            <div class="detail-row"><span class="detail-label">PID</span><span class="detail-value">${entry.pid}</span></div>
            <div class="detail-row"><span class="detail-label">Name</span><span class="detail-value">${escapeHtml(entry.process_name)}</span></div>
            <div class="detail-row"><span class="detail-label">Command</span><span class="detail-value" style="word-break: break-all;">${escapeHtml(entry.command)}</span></div>
            <div class="detail-row"><span class="detail-label">Memory</span><span class="detail-value">${entry.memory_mb.toFixed(1)} MB</span></div>
            <div class="detail-row"><span class="detail-label">CPU</span><span class="detail-value">${entry.cpu_percent.toFixed(1)}%</span></div>
        </div>`;

    if (entry.project) {
        html += `
        <div class="detail-section">
            <h3>📦 Project</h3>
            <div class="detail-row"><span class="detail-label">Kind</span><span class="detail-value">${escapeHtml(entry.project.kind)}</span></div>
            ${entry.project.framework ? `<div class="detail-row"><span class="detail-label">Framework</span><span class="detail-value" style="color: var(--accent-blue);">${escapeHtml(entry.project.framework)}</span></div>` : ''}
            ${entry.project.version ? `<div class="detail-row"><span class="detail-label">Version</span><span class="detail-value">${escapeHtml(entry.project.version)}</span></div>` : ''}
        </div>`;
    }

    if (entry.git) {
        html += `
        <div class="detail-section">
            <h3>🔀 Git</h3>
            <div class="detail-row"><span class="detail-label">Branch</span><span class="detail-value" style="color: ${entry.git.dirty ? 'var(--warning)' : 'var(--healthy)'};">${escapeHtml(entry.git.branch)}${entry.git.dirty ? ' *' : ''}</span></div>
            <div class="detail-row"><span class="detail-label">Status</span><span class="detail-value">${entry.git.dirty ? 'Modified' : 'Clean'}</span></div>
        </div>`;
    }

    if (entry.docker) {
        html += `
        <div class="detail-section">
            <h3>🐳 Docker</h3>
            <div class="detail-row"><span class="detail-label">Container</span><span class="detail-value" style="color: var(--accent-cyan);">${escapeHtml(entry.docker.container_name)}</span></div>
            <div class="detail-row"><span class="detail-label">Image</span><span class="detail-value">${escapeHtml(entry.docker.image)}</span></div>
            ${entry.docker.compose_project ? `<div class="detail-row"><span class="detail-label">Compose</span><span class="detail-value">${escapeHtml(entry.docker.compose_project)}</span></div>` : ''}
        </div>`;
    }

    if (entry.health_check) {
        html += `
        <div class="detail-section">
            <h3>🏥 Health</h3>
            <div class="detail-row"><span class="detail-label">Status</span><span class="detail-value">${getHealthText(entry.health_check.status)}</span></div>
            ${entry.health_check.status_code ? `<div class="detail-row"><span class="detail-label">HTTP Code</span><span class="detail-value">${entry.health_check.status_code}</span></div>` : ''}
            <div class="detail-row"><span class="detail-label">Latency</span><span class="detail-value">${entry.health_check.latency_ms}ms</span></div>
        </div>`;
    }

    html += `
        <div style="margin-top: 24px; display: flex; gap: 8px;">
            <button onclick="killPort(${entry.port}); closeModal();"
                    style="padding: 8px 20px; background: rgba(248, 81, 73, 0.15); border: 1px solid rgba(248, 81, 73, 0.3); color: var(--error); border-radius: 6px; cursor: pointer; font-size: 0.85rem; font-weight: 500; transition: all 150ms;">
                Kill Process
            </button>
            <button onclick="closeModal()"
                    style="padding: 8px 20px; background: var(--bg-highlight); border: 1px solid var(--border); color: var(--text-secondary); border-radius: 6px; cursor: pointer; font-size: 0.85rem; font-weight: 500; transition: all 150ms;">
                Close
            </button>
        </div>`;

    return html;
}

// ─── Kill Port ───

async function killPort(port) {
    if (!confirm(`Kill the process on port ${port}?`)) return;

    try {
        const response = await fetch(`/api/ports/${port}/kill`, { method: 'POST' });
        const result = await response.json();

        if (result.status === 'ok') {
            showToast(`✓ ${result.message}`, 'success');
            // Trigger HTMX refresh
            htmx.trigger('#port-table', 'htmx:load');
            htmx.trigger('#stats-section', 'htmx:load');
        } else {
            showToast(`✗ ${result.message}`, 'error');
        }
    } catch (err) {
        showToast(`✗ Failed to kill port ${port}`, 'error');
    }
}

// ─── Search / Filter ───

function filterTable(query) {
    const rows = document.querySelectorAll('.port-row');
    const q = query.toLowerCase();

    rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(q) ? '' : 'none';
    });
}

// ─── Toast Notifications ───

function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.textContent = message;

    Object.assign(toast.style, {
        position: 'fixed',
        bottom: '24px',
        right: '24px',
        padding: '12px 24px',
        borderRadius: '8px',
        fontSize: '0.85rem',
        fontWeight: '500',
        zIndex: '1000',
        animation: 'toast-in 200ms ease',
        background: type === 'success' ? 'rgba(63, 185, 80, 0.15)' : 'rgba(248, 81, 73, 0.15)',
        border: `1px solid ${type === 'success' ? 'rgba(63, 185, 80, 0.3)' : 'rgba(248, 81, 73, 0.3)'}`,
        color: type === 'success' ? '#3fb950' : '#f85149',
        backdropFilter: 'blur(16px)',
    });

    document.body.appendChild(toast);
    setTimeout(() => {
        toast.style.animation = 'toast-out 200ms ease forwards';
        setTimeout(() => toast.remove(), 200);
    }, 3000);
}

// ─── Helpers ───

function getStatusClass(status) {
    if (typeof status === 'string') {
        return `status-${status.toLowerCase()}`;
    }
    if (status === 'Healthy') return 'status-healthy';
    if (status === 'Zombie') return 'status-error';
    if (status === 'Orphaned') return 'status-warning';
    if (status && status.Warning) return 'status-warning';
    return 'status-unknown';
}

function getStatusText(status) {
    if (typeof status === 'string') return status;
    if (status && status.Warning) return `⚠ ${status.Warning}`;
    return status || 'Unknown';
}

function getHealthText(status) {
    if (status === 'Healthy') return '✓ Healthy';
    if (status === 'Unhealthy') return '✗ Unhealthy';
    return '? Unknown';
}

function escapeHtml(str) {
    if (!str) return '';
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// ─── Toast Animation Styles ───

const style = document.createElement('style');
style.textContent = `
@keyframes toast-in {
    from { opacity: 0; transform: translateY(16px); }
    to { opacity: 1; transform: translateY(0); }
}
@keyframes toast-out {
    from { opacity: 1; transform: translateY(0); }
    to { opacity: 0; transform: translateY(16px); }
}`;
document.head.appendChild(style);
