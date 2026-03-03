// TOR Web Capture - Client-side JavaScript

// Toast notification system
function showToast(message, type = 'info') {
    const container = document.getElementById('toast-container') || createToastContainer();

    const toast = document.createElement('div');
    toast.className = `alert alert-${type} toast`;

    const span = document.createElement('span');
    span.textContent = message;
    toast.appendChild(span);

    container.appendChild(toast);

    setTimeout(() => {
        toast.remove();
    }, 5000);
}

function createToastContainer() {
    const container = document.createElement('div');
    container.id = 'toast-container';
    container.className = 'toast-container';
    document.body.appendChild(container);
    return container;
}

// HTMX event handlers
document.addEventListener('htmx:afterRequest', function(event) {
    const xhr = event.detail.xhr;

    // Check for HX-Trigger header
    const trigger = xhr.getResponseHeader('HX-Trigger');
    if (trigger) {
        const triggers = trigger.split(',').map(t => t.trim());
        triggers.forEach(t => {
            switch(t) {
                case 'capture-started':
                    showToast('Capture started', 'info');
                    break;
                case 'capture-completed':
                    showToast('Capture completed', 'success');
                    break;
                case 'target-created':
                    showToast('Target created', 'success');
                    break;
                case 'target-updated':
                    showToast('Target updated', 'success');
                    break;
                case 'target-deleted':
                    showToast('Target deleted', 'warning');
                    break;
                case 'schedule-created':
                    showToast('Schedule created', 'success');
                    break;
                case 'settings-updated':
                    showToast('Settings saved', 'success');
                    break;
            }
        });
    }
});

document.addEventListener('htmx:responseError', function(event) {
    showToast('An error occurred: ' + event.detail.xhr.statusText, 'error');
});

// Cron expression helper
function updateCronPreview(expression) {
    const preview = document.getElementById('cron-preview');
    if (!preview) return;

    try {
        // Simple cron description (basic implementation)
        const parts = expression.split(' ');
        if (parts.length === 5) {
            const [min, hour, dom, mon, dow] = parts;
            let desc = 'Runs ';

            if (min === '*' && hour === '*') {
                desc += 'every minute';
            } else if (min === '0' && hour === '*') {
                desc += 'every hour';
            } else if (hour !== '*' && min !== '*') {
                desc += 'at ' + hour + ':' + min.padStart(2, '0');
            }

            preview.textContent = desc;
        }
    } catch (e) {
        preview.textContent = 'Invalid expression';
    }
}

// Auto-refresh for running captures
function setupAutoRefresh() {
    const runningCaptures = document.querySelectorAll('[data-status="running"]');
    runningCaptures.forEach(el => {
        if (!el.hasAttribute('hx-trigger')) {
            el.setAttribute('hx-get', el.dataset.refreshUrl);
            el.setAttribute('hx-trigger', 'every 5s');
            el.setAttribute('hx-swap', 'outerHTML');
            htmx.process(el);
        }
    });
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    setupAutoRefresh();
});

// Re-initialize after HTMX swaps
document.addEventListener('htmx:afterSwap', function() {
    setupAutoRefresh();
});

// Confirm dialogs
document.addEventListener('htmx:confirm', function(event) {
    if (event.detail.question) {
        if (!confirm(event.detail.question)) {
            event.preventDefault();
        }
    }
});

// Copy to clipboard helper
function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        showToast('Copied to clipboard', 'success');
    }).catch(() => {
        showToast('Failed to copy', 'error');
    });
}
