// Print tabanlı PDF dışa aktarım yardımcısı.
// Yeni pencere açar, tarayıcının Print → "PDF olarak kaydet" özelliğini kullanır.
// Türkçe karakterler (ğ, ş, İ, ü, ç...) tarayıcı fontlarıyla sorunsuz çalışır.
// Tauri WebView'de de güvenilir şekilde çalışır.

export type PdfTableSection = {
	kind: 'table';
	heading?: string;
	columns: string[];
	rows: Array<Array<string | number>>;
	widths?: Array<string | number>;
};

export type PdfKVSection = {
	kind: 'kv';
	heading?: string;
	items: Array<{ label: string; value: string | number }>;
	columns?: 1 | 2;
};

export type PdfTextSection = {
	kind: 'text';
	heading?: string;
	text: string;
};

export type PdfSection = PdfTableSection | PdfKVSection | PdfTextSection;

export type PdfExportOptions = {
	title: string;
	subtitle?: string;
	fileName: string;
	sections: PdfSection[];
	appName?: string;
	landscape?: boolean;
};

// ─── Formatlayıcılar ────────────────────────────────────────────────────────

export function formatTL(n: number): string {
	return new Intl.NumberFormat('tr-TR', {
		style: 'currency',
		currency: 'TRY',
		minimumFractionDigits: 2
	}).format(n);
}

export function formatTarih(t?: string | null): string {
	if (!t) return '-';
	try {
		return new Date(t).toLocaleDateString('tr-TR', {
			day: '2-digit',
			month: '2-digit',
			year: 'numeric'
		});
	} catch {
		return t;
	}
}

// ─── HTML Üretim ────────────────────────────────────────────────────────────

function esc(s: unknown): string {
	return String(s ?? '')
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;');
}

function sectionHtml(section: PdfSection): string {
	const heading = section.heading
		? `<h2 class="section-heading">${esc(section.heading)}</h2>`
		: '';

	if (section.kind === 'table') {
		const thead = `<thead><tr>${section.columns.map((c) => `<th>${esc(c)}</th>`).join('')}</tr></thead>`;
		const tbody = `<tbody>${section.rows
			.map((r) => `<tr>${r.map((c) => `<td>${esc(c)}</td>`).join('')}</tr>`)
			.join('')}</tbody>`;
		return `${heading}<table class="data-table">${thead}${tbody}</table>`;
	}

	if (section.kind === 'kv') {
		const cols = section.columns ?? 2;
		if (cols === 1) {
			const rows = section.items
				.map(
					(it) =>
						`<tr><th class="kv-label">${esc(it.label)}</th><td>${esc(it.value ?? '-')}</td></tr>`
				)
				.join('');
			return `${heading}<table class="kv-table kv-1col">${rows}</table>`;
		}
		const rowsHtml: string[] = [];
		for (let i = 0; i < section.items.length; i += 2) {
			const a = section.items[i];
			const b = section.items[i + 1];
			rowsHtml.push(
				`<tr>` +
					`<th class="kv-label">${esc(a?.label ?? '')}</th>` +
					`<td>${esc(a?.value ?? '-')}</td>` +
					`<th class="kv-label">${esc(b?.label ?? '')}</th>` +
					`<td>${b ? esc(b.value ?? '-') : ''}</td>` +
					`</tr>`
			);
		}
		return `${heading}<table class="kv-table kv-2col">${rowsHtml.join('')}</table>`;
	}

	return `${heading}<p class="text-section">${esc(section.text)}</p>`;
}

function buildHtml(opts: PdfExportOptions): string {
	const now = new Date().toLocaleString('tr-TR');
	const appName = opts.appName ?? 'Kooperatif Yönetim';
	const orientation = opts.landscape ? 'landscape' : 'portrait';
	const body = opts.sections.map(sectionHtml).join('\n');

	return `<!DOCTYPE html>
<html lang="tr">
<head>
<meta charset="UTF-8">
<title>${esc(opts.fileName)}</title>
<style>
	@page { size: A4 ${orientation}; margin: 14mm 12mm 16mm 12mm; }
	* { box-sizing: border-box; }
	html, body {
		margin: 0; padding: 0;
		font-family: 'Segoe UI', Tahoma, Verdana, Arial, sans-serif;
		font-size: 10pt; color: #111827;
		-webkit-print-color-adjust: exact;
		print-color-adjust: exact;
	}
	.container { padding: 12px 16px; }
	.page-header {
		display: flex; justify-content: space-between; align-items: flex-end;
		border-bottom: 2px solid #1e3a8a; padding-bottom: 6px; margin-bottom: 10px;
	}
	.page-header .app { font-size: 9pt; color: #6b7280; }
	.page-header .meta { font-size: 8pt; color: #6b7280; }
	h1.title { margin: 0 0 2px 0; font-size: 18pt; color: #1e3a8a; }
	.subtitle { margin: 0 0 10px 0; font-size: 11pt; color: #374151; }
	.section-heading {
		margin: 14px 0 6px 0; font-size: 12pt; color: #1f2937;
		border-left: 3px solid #4f46e5; padding-left: 8px;
	}
	table { width: 100%; border-collapse: collapse; font-size: 9.5pt; }
	table.data-table { margin-bottom: 8px; }
	table.data-table th, table.data-table td {
		border: 0.5px solid #e5e7eb; padding: 5px 7px; text-align: left; vertical-align: top;
	}
	table.data-table thead th {
		background: #eef2ff; color: #111827; font-weight: 600;
	}
	table.data-table tbody tr:nth-child(even) td { background: #f9fafb; }
	table.kv-table th, table.kv-table td {
		padding: 4px 8px; border-bottom: 0.5px solid #e5e7eb; vertical-align: top;
	}
	table.kv-table .kv-label { color: #374151; font-weight: 600; width: 22%; text-align: left; }
	.text-section { margin: 4px 0 10px 0; white-space: pre-wrap; }
	.footer-note {
		margin-top: 14px; padding-top: 6px; border-top: 0.5px solid #e5e7eb;
		font-size: 8pt; color: #6b7280; text-align: center;
	}
	@media print {
		.no-print { display: none !important; }
		thead { display: table-header-group; }
		tr { page-break-inside: avoid; }
	}
	.action-bar {
		position: sticky; top: 0; z-index: 10;
		background: #1e3a8a; color: #fff; padding: 10px 16px;
		display: flex; justify-content: space-between; align-items: center;
	}
	.action-bar button {
		background: #fff; color: #1e3a8a; border: 0; padding: 6px 14px;
		font-size: 10pt; font-weight: 600; border-radius: 4px; cursor: pointer;
		margin-left: 8px;
	}
	.action-bar button:hover { background: #eef2ff; }
	.action-bar .close { background: transparent; color: #fff; border: 1px solid #fff; }
</style>
</head>
<body>
	<div class="action-bar no-print">
		<span>PDF Önizleme — "Yazdır / PDF Kaydet" ile indirebilirsiniz</span>
		<div>
			<button onclick="window.print()">Yazdır / PDF Kaydet</button>
			<button class="close" onclick="window.close()">Kapat</button>
		</div>
	</div>
	<div class="container">
		<div class="page-header">
			<div class="app">${esc(appName)}</div>
			<div class="meta">${esc(now)}</div>
		</div>
		<h1 class="title">${esc(opts.title)}</h1>
		${opts.subtitle ? `<div class="subtitle">${esc(opts.subtitle)}</div>` : ''}
		${body}
		<div class="footer-note">${esc(appName)} • ${esc(now)}</div>
	</div>
	<script>
		window.addEventListener('load', () => {
			setTimeout(() => { try { window.print(); } catch (e) {} }, 400);
		});
	</script>
</body>
</html>`;
}

// ─── Dışa Aktarım ───────────────────────────────────────────────────────────

export function exportPdf(opts: PdfExportOptions): void {
	const html = buildHtml(opts);

	// Yeni pencerede aç
	const win = window.open('', '_blank', 'width=900,height=1100');
	if (win) {
		win.document.open();
		win.document.write(html);
		win.document.close();
		return;
	}

	// Pencere açılamadıysa iframe ile yazdır
	try {
		const iframe = document.createElement('iframe');
		iframe.style.position = 'fixed';
		iframe.style.right = '0';
		iframe.style.bottom = '0';
		iframe.style.width = '0';
		iframe.style.height = '0';
		iframe.style.border = '0';
		document.body.appendChild(iframe);

		const doc = iframe.contentDocument || iframe.contentWindow?.document;
		if (!doc) throw new Error('iframe document erişilemiyor');
		doc.open();
		doc.write(html);
		doc.close();

		iframe.onload = () => {
			setTimeout(() => {
				try {
					iframe.contentWindow?.focus();
					iframe.contentWindow?.print();
				} catch (e) {
					console.error('[pdf] iframe print hatası:', e);
				}
				setTimeout(() => {
					try {
						document.body.removeChild(iframe);
					} catch {}
				}, 2000);
			}, 500);
		};
	} catch (e) {
		console.error('[pdf] Oluşturulamadı:', e);
		alert('PDF oluşturulamadı. Pop-up engelleyici aktif olabilir.');
	}
}
