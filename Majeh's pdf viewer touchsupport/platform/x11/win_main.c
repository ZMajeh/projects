#include "pdfapp.h"

#ifndef UNICODE
#define UNICODE
#endif
#ifndef _UNICODE
#define _UNICODE
#endif
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <commdlg.h>
#include <shellapi.h>

#ifndef WM_MOUSEWHEEL
#define WM_MOUSEWHEEL 0x020A
#endif

#ifndef WM_GESTURE
#define WM_GESTURE 0x0119
#endif
#ifndef WM_TOUCH
#define WM_TOUCH 0x0240
#endif
#ifndef TWF_WANTPALM
#define TWF_WANTPALM 0x00000002
#endif
#ifndef TOUCHEVENTF_DOWN
#define TOUCHEVENTF_DOWN 0x0002
#endif
#ifndef TOUCHEVENTF_MOVE
#define TOUCHEVENTF_MOVE 0x0001
#endif
#ifndef TOUCHEVENTF_UP
#define TOUCHEVENTF_UP 0x0004
#endif
#ifndef GID_ZOOM
#define GID_ZOOM 3
#endif
#ifndef GF_BEGIN
#define GF_BEGIN 0x00000001
#endif

typedef struct {
	DWORD id;
	int x, y;
	int start_x, start_y;
	int active;
} touch_point_t;

static touch_point_t touch_pts[10];
static int gesture_fired = 0;
static int touch_max_active = 0;

#define MIN(x,y) ((x) < (y) ? (x) : (y))

#define ID_ABOUT	0x1000
#define ID_DOCINFO	0x1001

static HWND hwndframe = NULL;
static HWND hwndview = NULL;
static HDC hdc;
static HBRUSH bgbrush;
static HBRUSH shbrush;
static BITMAPINFO *dibinf = NULL;
static HCURSOR arrowcurs, handcurs, waitcurs, caretcurs;
static LRESULT CALLBACK frameproc(HWND, UINT, WPARAM, LPARAM);
static LRESULT CALLBACK viewproc(HWND, UINT, WPARAM, LPARAM);
static int timer_pending = 0;

static int justcopied = 0;

static pdfapp_t gapp;

#ifndef PATH_MAX
#define PATH_MAX (1024)
#endif

static wchar_t wbuf[PATH_MAX];
static char filename[PATH_MAX];

/*
 * Create registry keys to associate Majeh's PDF Viewer with PDF and XPS files.
 */

#define OPEN_KEY(parent, name, ptr) \
	RegCreateKeyExA(parent, name, 0, 0, 0, KEY_WRITE, 0, &ptr, 0)

#define SET_KEY(parent, name, value) \
	RegSetValueExA(parent, name, 0, REG_SZ, (const BYTE *)(value), strlen(value) + 1)

void install_app(char *argv0)
{
	char buf[512];
	HKEY software, classes, majehviewer, dotpdf, dotxps;
	HKEY shell, open, command, supported_types;
	HKEY pdf_progids, xps_progids;

	OPEN_KEY(HKEY_CURRENT_USER, "Software", software);
	OPEN_KEY(software, "Classes", classes);
	OPEN_KEY(classes, ".pdf", dotpdf);
	OPEN_KEY(dotpdf, "OpenWithProgids", pdf_progids);
	OPEN_KEY(classes, ".xps", dotxps);
	OPEN_KEY(dotxps, "OpenWithProgids", xps_progids);
	OPEN_KEY(classes, "Majeh's PDF Viewer", majehviewer);
	OPEN_KEY(majehviewer, "SupportedTypes", supported_types);
	OPEN_KEY(majehviewer, "shell", shell);
	OPEN_KEY(shell, "open", open);
	OPEN_KEY(open, "command", command);

	sprintf(buf, "\"%s\" \"%%1\"", argv0);

	SET_KEY(open, "FriendlyAppName", "Majeh's PDF Viewer");
	SET_KEY(command, "", buf);
	SET_KEY(supported_types, ".pdf", "");
	SET_KEY(supported_types, ".xps", "");
	SET_KEY(pdf_progids, "Majeh's PDF Viewer", "");
	SET_KEY(xps_progids, "Majeh's PDF Viewer", "");

	RegCloseKey(dotxps);
	RegCloseKey(dotpdf);
	RegCloseKey(majehviewer);
	RegCloseKey(classes);
	RegCloseKey(software);
}

/*
 * Dialog boxes
 */

void winwarn(pdfapp_t *app, char *msg)
{
	MessageBoxA(hwndframe, msg, "Majeh's PDF Viewer: Warning", MB_ICONWARNING);
}

int winquery(pdfapp_t *app, char *msg)
{
	return MessageBoxA(hwndframe, msg, "Majeh's PDF Viewer", MB_YESNO | MB_ICONQUESTION) == IDYES;
}

void winerror(pdfapp_t *app, char *msg)
{
	MessageBoxA(hwndframe, msg, "Majeh's PDF Viewer: Error", MB_ICONERROR);
	exit(1);
}

void winalert(pdfapp_t *app, pdf_alert_event *alert)
{
	int buttons = MB_OK;
	int icon = MB_ICONWARNING;
	int pressed = PDF_ALERT_BUTTON_NONE;

	switch (alert->icon_type)
	{
	case PDF_ALERT_ICON_ERROR:
		icon = MB_ICONERROR;
		break;
	case PDF_ALERT_ICON_WARNING:
		icon = MB_ICONWARNING;
		break;
	case PDF_ALERT_ICON_QUESTION:
		icon = MB_ICONQUESTION;
		break;
	case PDF_ALERT_ICON_STATUS:
		icon = MB_ICONINFORMATION;
		break;
	}

	switch (alert->button_group_type)
	{
	case PDF_ALERT_BUTTON_GROUP_OK:
		buttons = MB_OK;
		break;
	case PDF_ALERT_BUTTON_GROUP_OK_CANCEL:
		buttons = MB_OKCANCEL;
		break;
	case PDF_ALERT_BUTTON_GROUP_YES_NO:
		buttons = MB_YESNO;
		break;
	case PDF_ALERT_BUTTON_GROUP_YES_NO_CANCEL:
		buttons = MB_YESNOCANCEL;
		break;
	}

	pressed = MessageBoxA(hwndframe, alert->message, alert->title, icon|buttons);

	switch (pressed)
	{
	case IDOK:
		alert->button_pressed = PDF_ALERT_BUTTON_OK;
		break;
	case IDCANCEL:
		alert->button_pressed = PDF_ALERT_BUTTON_CANCEL;
		break;
	case IDNO:
		alert->button_pressed = PDF_ALERT_BUTTON_NO;
		break;
	case IDYES:
		alert->button_pressed = PDF_ALERT_BUTTON_YES;
	}
}

static int pd_okay = 0;
static int print_range_type = 0; /* 0=All, 1=Current, 2=Custom */
static char print_range_text[256] = "";
static int print_dpi_choice = 0; /* 0=Default, 1=72, 2=150, 3=300, 4=600 */

static int check_range_match(int p, const char *str)
{
	char buf[256];
	char *token;
	fz_strlcpy(buf, str, sizeof(buf));
	token = strtok(buf, ",");
	while (token)
	{
		while (*token == ' ') token++;
		if (strchr(token, '-'))
		{
			int start = atoi(token);
			int end = atoi(strchr(token, '-') + 1);
			if (p >= start && p <= end)
				return 1;
		}
		else
		{
			int num = atoi(token);
			if (p == num)
				return 1;
		}
		token = strtok(NULL, ",");
	}
	return 0;
}

static int is_page_included(int p, int pagecount)
{
	if (print_range_type == 0)
		return 1;
	if (print_range_type == 1)
		return (p == gapp.pageno);
	if (print_range_type == 2)
		return check_range_match(p, print_range_text);
	return 1;
}

static int get_dpi_value(int choice, int current_res)
{
	switch (choice)
	{
	case 1: return 72;
	case 2: return 150;
	case 3: return 300;
	case 4: return 600;
	default: return current_res > 0 ? current_res : 72;
	}
}

static void update_preview_status(HWND hwnd, int curr_page)
{
	char buf[64];
	int total_matched = 0;
	int p;
	for (p = 1; p <= gapp.pagecount; p++)
	{
		if (is_page_included(p, gapp.pagecount))
			total_matched++;
	}
	snprintf(buf, sizeof(buf), "Page %d (Total: %d)", curr_page, total_matched);
	SetDlgItemTextA(hwnd, 12, buf);
}

INT_PTR CALLBACK
dlogpreviewproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	static int curr_page = 1;
	switch(message)
	{
	case WM_INITDIALOG:
		{
			int p;
			curr_page = gapp.pageno;
			for (p = 1; p <= gapp.pagecount; p++)
			{
				if (is_page_included(p, gapp.pagecount))
				{
					curr_page = p;
					break;
				}
			}
			update_preview_status(hwnd, curr_page);
			return TRUE;
		}
	case WM_COMMAND:
		switch(wParam)
		{
		case 1:
			EndDialog(hwnd, 1);
			return TRUE;
		case 10: /* Prev */
			{
				int p;
				for (p = curr_page - 1; p >= 1; p--)
				{
					if (is_page_included(p, gapp.pagecount))
					{
						curr_page = p;
						break;
					}
				}
				update_preview_status(hwnd, curr_page);
				InvalidateRect(hwnd, NULL, TRUE);
			}
			return TRUE;
		case 11: /* Next */
			{
				int p;
				for (p = curr_page + 1; p <= gapp.pagecount; p++)
				{
					if (is_page_included(p, gapp.pagecount))
					{
						curr_page = p;
						break;
					}
				}
				update_preview_status(hwnd, curr_page);
				InvalidateRect(hwnd, NULL, TRUE);
			}
			return TRUE;
		}
		break;
	case WM_PAINT:
		{
			PAINTSTRUCT ps;
			HDC hdc = BeginPaint(hwnd, &ps);
			int old_pageno = gapp.pageno;
			int old_res = gapp.resolution;
			int dpi = get_dpi_value(print_dpi_choice, old_res);
			gapp.resolution = dpi > 0 ? dpi : 72;
			pdfapp_gotopage(&gapp, curr_page);

			if (gapp.image)
			{
				int image_w = fz_pixmap_width(gapp.ctx, gapp.image);
				int image_h = fz_pixmap_height(gapp.ctx, gapp.image);
				int image_n = fz_pixmap_components(gapp.ctx, gapp.image);
				unsigned char *samples = fz_pixmap_samples(gapp.ctx, gapp.image);
				RECT rc;
				GetClientRect(hwnd, &rc);
				int pw = rc.right - 20;
				int ph = rc.bottom - 60;
				int x = 10, y = 10, w = pw, h = ph;

				if (image_w > 0 && image_h > 0)
				{
					double scale_x = (double)pw / image_w;
					double scale_y = (double)ph / image_h;
					double scale = scale_x < scale_y ? scale_x : scale_y;
					w = (int)(image_w * scale);
					h = (int)(image_h * scale);
					x = 10 + (pw - w) / 2;
					y = 10 + (ph - h) / 2;
				}

				HBRUSH bg = CreateSolidBrush(RGB(0xa0, 0xa0, 0xa0));
				RECT prev_box = {10, 10, 10 + pw, 10 + ph};
				FillRect(hdc, &prev_box, bg);
				DeleteObject(bg);

				if (dibinf)
				{
					dibinf->bmiHeader.biWidth = image_w;
					dibinf->bmiHeader.biHeight = -image_h;
					dibinf->bmiHeader.biSizeImage = image_h * 4;

					if (image_n == 2)
					{
						int i = image_w * image_h;
						unsigned char *color = malloc(i * 4);
						if (color)
						{
							unsigned char *s = samples;
							unsigned char *d = color;
							for (; i > 0; i--)
							{
								d[2] = d[1] = d[0] = *s++;
								d[3] = *s++;
								d += 4;
							}
							StretchDIBits(hdc, x, y, w, h, 0, 0, image_w, image_h, color, dibinf, DIB_RGB_COLORS, SRCCOPY);
							free(color);
						}
					}
					else if (image_n == 4)
					{
						StretchDIBits(hdc, x, y, w, h, 0, 0, image_w, image_h, samples, dibinf, DIB_RGB_COLORS, SRCCOPY);
					}
				}
			}

			pdfapp_gotopage(&gapp, old_pageno);
			gapp.resolution = old_res;

			EndPaint(hwnd, &ps);
			return 0;
		}
	}
	return FALSE;
}

INT_PTR CALLBACK
dlogprintproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	HWND combo;
	switch(message)
	{
	case WM_INITDIALOG:
		CheckRadioButton(hwnd, 101, 103, 101 + print_range_type);
		SetDlgItemTextA(hwnd, 104, print_range_text);

		combo = GetDlgItem(hwnd, 105);
		SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"Default (Current)");
		SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"72 DPI (Low)");
		SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"150 DPI (Medium)");
		SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"300 DPI (High)");
		SendMessageA(combo, CB_ADDSTRING, 0, (LPARAM)"600 DPI (Very High)");
		SendMessageA(combo, CB_SETCURSEL, print_dpi_choice, 0);
		return TRUE;

	case WM_COMMAND:
		switch(wParam)
		{
		case 1:
			if (IsDlgButtonChecked(hwnd, 101) == BST_CHECKED)
				print_range_type = 0;
			else if (IsDlgButtonChecked(hwnd, 102) == BST_CHECKED)
				print_range_type = 1;
			else if (IsDlgButtonChecked(hwnd, 103) == BST_CHECKED)
				print_range_type = 2;

			GetDlgItemTextA(hwnd, 104, print_range_text, sizeof(print_range_text));
			combo = GetDlgItem(hwnd, 105);
			print_dpi_choice = SendMessageA(combo, CB_GETCURSEL, 0, 0);
			if (print_dpi_choice == CB_ERR)
				print_dpi_choice = 0;

			pd_okay = 1;
			EndDialog(hwnd, 1);
			return TRUE;

		case 3: /* Preview */
			if (IsDlgButtonChecked(hwnd, 101) == BST_CHECKED)
				print_range_type = 0;
			else if (IsDlgButtonChecked(hwnd, 102) == BST_CHECKED)
				print_range_type = 1;
			else if (IsDlgButtonChecked(hwnd, 103) == BST_CHECKED)
				print_range_type = 2;

			GetDlgItemTextA(hwnd, 104, print_range_text, sizeof(print_range_text));
			combo = GetDlgItem(hwnd, 105);
			print_dpi_choice = SendMessageA(combo, CB_GETCURSEL, 0, 0);
			if (print_dpi_choice == CB_ERR)
				print_dpi_choice = 0;

			DialogBoxW(NULL, L"IDD_DLOGPREVIEW", hwnd, dlogpreviewproc);
			return TRUE;

		case 2:
			pd_okay = 0;
			EndDialog(hwnd, 1);
			return TRUE;
		}
		break;
	}
	return FALSE;
}

void winprint(pdfapp_t *app)
{
	PRINTDLGA pd;
	DOCINFOA di;
	int old_pageno;
	int old_res;
	int dpi;
	int p;

	if (!app->doc)
	{
		winwarn(app, "No document loaded to print.");
		return;
	}

	pd_okay = 0;
	if (DialogBoxW(NULL, L"IDD_DLOGPRINT", hwndframe, dlogprintproc) <= 0 || !pd_okay)
		return;

	memset(&pd, 0, sizeof(pd));
	pd.lStructSize = sizeof(pd);
	pd.hwndOwner = hwndframe;
	pd.Flags = PD_RETURNDC | PD_NOPAGENUMS | PD_NOSELECTION;

	if (!PrintDlgA(&pd))
		return; /* User cancelled */

	memset(&di, 0, sizeof(di));
	di.cbSize = sizeof(di);
	di.lpszDocName = "Majeh's PDF Viewer Document";

	if (StartDocA(pd.hDC, &di) <= 0)
	{
		DeleteDC(pd.hDC);
		return;
	}

	old_pageno = app->pageno;
	old_res = app->resolution;
	dpi = get_dpi_value(print_dpi_choice, old_res);
	app->resolution = dpi > 0 ? dpi : 72;

	for (p = 1; p <= app->pagecount; p++)
	{
		int image_w, image_h, image_n;
		unsigned char *samples;
		unsigned char *color = NULL;
		BITMAPINFO *bmi;
		int pw, ph;
		int x, y, w, h;

		if (!is_page_included(p, app->pagecount))
			continue;

		pdfapp_gotopage(app, p);

		if (!app->image)
			continue;

		if (StartPage(pd.hDC) <= 0)
			break;

		image_w = fz_pixmap_width(app->ctx, app->image);
		image_h = fz_pixmap_height(app->ctx, app->image);
		image_n = fz_pixmap_components(app->ctx, app->image);
		samples = fz_pixmap_samples(app->ctx, app->image);

		pw = GetDeviceCaps(pd.hDC, HORZRES);
		ph = GetDeviceCaps(pd.hDC, VERTRES);

		/* Scale image to fit printer page while maintaining aspect ratio */
		if (image_w > 0 && image_h > 0)
		{
			double scale_x = (double)pw / image_w;
			double scale_y = (double)ph / image_h;
			double scale = scale_x < scale_y ? scale_x : scale_y;
			w = (int)(image_w * scale);
			h = (int)(image_h * scale);
			x = (pw - w) / 2;
			y = (ph - h) / 2;
		}
		else
		{
			w = pw;
			h = ph;
			x = y = 0;
		}

		bmi = malloc(sizeof(BITMAPINFO) + 12);
		if (bmi)
		{
			bmi->bmiHeader.biSize = sizeof(bmi->bmiHeader);
			bmi->bmiHeader.biWidth = image_w;
			bmi->bmiHeader.biHeight = -image_h;
			bmi->bmiHeader.biPlanes = 1;
			bmi->bmiHeader.biBitCount = 32;
			bmi->bmiHeader.biCompression = BI_RGB;
			bmi->bmiHeader.biSizeImage = image_h * 4;
			bmi->bmiHeader.biXPelsPerMeter = 2834;
			bmi->bmiHeader.biYPelsPerMeter = 2834;
			bmi->bmiHeader.biClrUsed = 0;
			bmi->bmiHeader.biClrImportant = 0;

			if (image_n == 2)
			{
				int i = image_w * image_h;
				color = malloc(i * 4);
				if (color)
				{
					unsigned char *s = samples;
					unsigned char *d = color;
					for (; i > 0; i--)
					{
						d[2] = d[1] = d[0] = *s++;
						d[3] = *s++;
						d += 4;
					}
					StretchDIBits(pd.hDC, x, y, w, h, 0, 0, image_w, image_h, color, bmi, DIB_RGB_COLORS, SRCCOPY);
					free(color);
				}
			}
			else if (image_n == 4)
			{
				StretchDIBits(pd.hDC, x, y, w, h, 0, 0, image_w, image_h, samples, bmi, DIB_RGB_COLORS, SRCCOPY);
			}
			free(bmi);
		}

		EndPage(pd.hDC);
	}

	pdfapp_gotopage(app, old_pageno);
	app->resolution = old_res;

	EndDoc(pd.hDC);
	DeleteDC(pd.hDC);
}

int winsavequery(pdfapp_t *app)
{
	switch(MessageBoxA(hwndframe, "File has unsaved changes. Do you want to save", "Majeh's PDF Viewer", MB_YESNOCANCEL))
	{
	case IDYES: return SAVE;
	case IDNO: return DISCARD;
	default: return CANCEL;
	}
}

int winfilename(wchar_t *buf, int len)
{
	OPENFILENAME ofn;
	buf[0] = 0;
	memset(&ofn, 0, sizeof(OPENFILENAME));
	ofn.lStructSize = sizeof(OPENFILENAME);
	ofn.hwndOwner = hwndframe;
	ofn.lpstrFile = buf;
	ofn.nMaxFile = len;
	ofn.lpstrInitialDir = NULL;
	ofn.lpstrTitle = L"Majeh's PDF Viewer: Open PDF file";
	ofn.lpstrFilter = L"Documents (*.pdf;*.xps;*.cbz;*.zip;*.png;*.jpg;*.tif;*.txt;*.reg)\0*.zip;*.cbz;*.xps;*.pdf;*.jpe;*.jpg;*.jpeg;*.jfif;*.tif;*.tiff;*.txt;*.reg\0PDF Files (*.pdf)\0*.pdf\0XPS Files (*.xps)\0*.xps\0CBZ Files (*.cbz;*.zip)\0*.zip;*.cbz\0Text Files (*.txt;*.reg)\0*.txt;*.reg\0Image Files (*.png;*.jpe;*.tif)\0*.png;*.jpg;*.jpe;*.jpeg;*.jfif;*.tif;*.tiff\0All Files\0*\0\0";
	ofn.Flags = OFN_FILEMUSTEXIST|OFN_HIDEREADONLY;
	return GetOpenFileNameW(&ofn);
}

int wingetsavepath(pdfapp_t *app, char *buf, int len)
{
	wchar_t twbuf[PATH_MAX];
	OPENFILENAME ofn;

	wcscpy(twbuf, wbuf);
	memset(&ofn, 0, sizeof(OPENFILENAME));
	ofn.lStructSize = sizeof(OPENFILENAME);
	ofn.hwndOwner = hwndframe;
	ofn.lpstrFile = twbuf;
	ofn.nMaxFile = PATH_MAX;
	ofn.lpstrInitialDir = NULL;
	ofn.lpstrTitle = L"Majeh's PDF Viewer: Save PDF file";
	ofn.lpstrFilter = L"Documents (*.pdf;*.xps;*.cbz;*.zip;*.png;*.jpg;*.tif;*.txt;*.reg)\0*.zip;*.cbz;*.xps;*.pdf;*.jpe;*.jpg;*.jpeg;*.jfif;*.tif;*.tiff;*.txt;*.reg\0PDF Files (*.pdf)\0*.pdf\0XPS Files (*.xps)\0*.xps\0CBZ Files (*.cbz;*.zip)\0*.zip;*.cbz\0Text Files (*.txt;*.reg)\0*.txt;*.reg\0Image Files (*.png;*.jpe;*.tif)\0*.png;*.jpg;*.jpe;*.jpeg;*.jfif;*.tif;*.tiff\0All Files\0*\0\0";
	ofn.Flags = OFN_HIDEREADONLY;
	if (GetSaveFileName(&ofn))
	{
		int code = WideCharToMultiByte(CP_UTF8, 0, twbuf, -1, buf, MIN(PATH_MAX, len), NULL, NULL);
		if (code == 0)
		{
			winerror(&gapp, "cannot convert filename to utf-8");
			return 0;
		}

		wcscpy(wbuf, twbuf);
		strcpy(filename, buf);
		return 1;
	}
	else
	{
		return 0;
	}
}

void winreplacefile(char *source, char *target)
{
	wchar_t wsource[PATH_MAX];
	wchar_t wtarget[PATH_MAX];

	int sz = MultiByteToWideChar(CP_UTF8, 0, source, -1, wsource, PATH_MAX);
	if (sz == 0)
	{
		winerror(&gapp, "cannot convert filename to Unicode");
		return;
	}

	sz = MultiByteToWideChar(CP_UTF8, 0, target, -1, wtarget, PATH_MAX);
	if (sz == 0)
	{
		winerror(&gapp, "cannot convert filename to Unicode");
		return;
	}

#if (_WIN32_WINNT >= 0x0500)
	ReplaceFile(wtarget, wsource, NULL, REPLACEFILE_IGNORE_MERGE_ERRORS, NULL, NULL);
#else
	DeleteFile(wtarget);
	MoveFile(wsource, wtarget);
#endif
}

void wincopyfile(char *source, char *target)
{
	wchar_t wsource[PATH_MAX];
	wchar_t wtarget[PATH_MAX];

	int sz = MultiByteToWideChar(CP_UTF8, 0, source, -1, wsource, PATH_MAX);
	if (sz == 0)
	{
		winerror(&gapp, "cannot convert filename to Unicode");
		return;
	}

	sz = MultiByteToWideChar(CP_UTF8, 0, target, -1, wtarget, PATH_MAX);
	if (sz == 0)
	{
		winerror(&gapp, "cannot convert filename to Unicode");
		return;
	}

	CopyFile(wsource, wtarget, FALSE);
}

static char pd_filename[256] = "The file is encrypted.";
static char pd_password[256] = "";
static wchar_t pd_passwordw[256] = {0};
static char td_textinput[1024] = "";
static int td_retry = 0;
static int cd_nopts = 0;
static int *cd_nvals = NULL;
static char **cd_opts = NULL;
static char **cd_vals = NULL;




INT_PTR CALLBACK
dlogpassproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch(message)
	{
	case WM_INITDIALOG:
		SetDlgItemTextA(hwnd, 4, pd_filename);
		return TRUE;
	case WM_COMMAND:
		switch(wParam)
		{
		case 1:
			pd_okay = 1;
			GetDlgItemTextW(hwnd, 3, pd_passwordw, nelem(pd_passwordw));
			EndDialog(hwnd, 1);
			WideCharToMultiByte(CP_UTF8, 0, pd_passwordw, -1, pd_password, sizeof pd_password, NULL, NULL);
			return TRUE;
		case 2:
			pd_okay = 0;
			EndDialog(hwnd, 1);
			return TRUE;
		}
		break;
	}
	return FALSE;
}

INT_PTR CALLBACK
dlogtextproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch(message)
	{
	case WM_INITDIALOG:
		SetDlgItemTextA(hwnd, 3, td_textinput);
		if (!td_retry)
			ShowWindow(GetDlgItem(hwnd, 4), SW_HIDE);
		return TRUE;
	case WM_COMMAND:
		switch(wParam)
		{
		case 1:
			pd_okay = 1;
			GetDlgItemTextA(hwnd, 3, td_textinput, sizeof td_textinput);
			EndDialog(hwnd, 1);
			return TRUE;
		case 2:
			pd_okay = 0;
			EndDialog(hwnd, 1);
			return TRUE;
		}
		break;
	case WM_CTLCOLORSTATIC:
		if ((HWND)lParam == GetDlgItem(hwnd, 4))
		{
			SetTextColor((HDC)wParam, RGB(255,0,0));
			SetBkMode((HDC)wParam, TRANSPARENT);

			return (INT_PTR)GetStockObject(NULL_BRUSH);
		}
		break;
	}
	return FALSE;
}

INT_PTR CALLBACK
dlogchoiceproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	HWND listbox;
	int i;
	int item;
	int sel;
	switch(message)
	{
	case WM_INITDIALOG:
		listbox = GetDlgItem(hwnd, 3);
		for (i = 0; i < cd_nopts; i++)
			SendMessageA(listbox, LB_ADDSTRING, 0, (LPARAM)cd_opts[i]);

		/* FIXME: handle multiple select */
		if (*cd_nvals > 0)
		{
			item = SendMessageA(listbox, LB_FINDSTRINGEXACT, (WPARAM)-1, (LPARAM)cd_vals[0]);
			if (item != LB_ERR)
				SendMessageA(listbox, LB_SETCURSEL, item, 0);
		}
		return TRUE;
	case WM_COMMAND:
		switch(wParam)
		{
		case 1:
			listbox = GetDlgItem(hwnd, 3);
			*cd_nvals = 0;
			for (i = 0; i < cd_nopts; i++)
			{
				item = SendMessageA(listbox, LB_FINDSTRINGEXACT, (WPARAM)-1, (LPARAM)cd_opts[i]);
				sel = SendMessageA(listbox, LB_GETSEL, item, 0);
				if (sel && sel != LB_ERR)
					cd_vals[(*cd_nvals)++] = cd_opts[i];
			}
			pd_okay = 1;
			EndDialog(hwnd, 1);
			return TRUE;
		case 2:
			pd_okay = 0;
			EndDialog(hwnd, 1);
			return TRUE;
		}
		break;
	}
	return FALSE;
}

char *winpassword(pdfapp_t *app, char *filename)
{
	char buf[1024], *s;
	int code;
	strcpy(buf, filename);
	s = buf;
	if (strrchr(s, '\\')) s = strrchr(s, '\\') + 1;
	if (strrchr(s, '/')) s = strrchr(s, '/') + 1;
	if (strlen(s) > 32)
		strcpy(s + 30, "...");
	snprintf(pd_filename, sizeof(pd_filename), "The file \"%.32s\" is encrypted.", s);
	code = DialogBoxW(NULL, L"IDD_DLOGPASS", hwndframe, dlogpassproc);
	if (code <= 0)
		winerror(app, "cannot create password dialog");
	if (pd_okay)
		return pd_password;
	return NULL;
}

char *wintextinput(pdfapp_t *app, char *inittext, int retry)
{
	int code;
	td_retry = retry;
	fz_strlcpy(td_textinput, inittext ? inittext : "", sizeof td_textinput);
	code = DialogBoxW(NULL, L"IDD_DLOGTEXT", hwndframe, dlogtextproc);
	if (code <= 0)
		winerror(app, "cannot create text input dialog");
	if (pd_okay)
		return td_textinput;
	return NULL;
}

int winchoiceinput(pdfapp_t *app, int nopts, char *opts[], int *nvals, char *vals[])
{
	int code;
	cd_nopts = nopts;
	cd_nvals = nvals;
	cd_opts = opts;
	cd_vals = vals;
	code = DialogBoxW(NULL, L"IDD_DLOGLIST", hwndframe, dlogchoiceproc);
	if (code <= 0)
		winerror(app, "cannot create text input dialog");
	return pd_okay;
}

INT_PTR CALLBACK
dloginfoproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	char buf[256];
	wchar_t bufx[256];
	fz_document *doc = gapp.doc;

	switch(message)
	{
	case WM_INITDIALOG:

		SetDlgItemTextW(hwnd, 0x10, wbuf);

		if (fz_meta(doc, FZ_META_FORMAT_INFO, buf, 256) < 0)
		{
			SetDlgItemTextA(hwnd, 0x11, "Unknown");
			SetDlgItemTextA(hwnd, 0x12, "None");
			SetDlgItemTextA(hwnd, 0x13, "n/a");
			return TRUE;
		}

		SetDlgItemTextA(hwnd, 0x11, buf);

		if (fz_meta(doc, FZ_META_CRYPT_INFO, buf, 256) == 0)
		{
			SetDlgItemTextA(hwnd, 0x12, buf);
		}
		else
		{
			SetDlgItemTextA(hwnd, 0x12, "None");
		}
		buf[0] = 0;
		if (fz_meta(doc, FZ_META_HAS_PERMISSION, NULL, FZ_PERMISSION_PRINT) == 0)
			strcat(buf, "print, ");
		if (fz_meta(doc, FZ_META_HAS_PERMISSION, NULL, FZ_PERMISSION_CHANGE) == 0)
			strcat(buf, "modify, ");
		if (fz_meta(doc, FZ_META_HAS_PERMISSION, NULL, FZ_PERMISSION_COPY) == 0)
			strcat(buf, "copy, ");
		if (fz_meta(doc, FZ_META_HAS_PERMISSION, NULL, FZ_PERMISSION_NOTES) == 0)
			strcat(buf, "annotate, ");
		if (strlen(buf) > 2)
			buf[strlen(buf)-2] = 0;
		else
			strcpy(buf, "None");
		SetDlgItemTextA(hwnd, 0x13, buf);

#define SETUTF8(ID, STRING) \
		{ \
			*(char **)buf = STRING; \
			if (fz_meta(doc, FZ_META_INFO, buf, 256) <= 0) \
				buf[0] = 0; \
			MultiByteToWideChar(CP_UTF8, 0, buf, -1, bufx, nelem(bufx)); \
			SetDlgItemTextW(hwnd, ID, bufx); \
		}

		SETUTF8(0x20, "Title");
		SETUTF8(0x21, "Author");
		SETUTF8(0x22, "Subject");
		SETUTF8(0x23, "Keywords");
		SETUTF8(0x24, "Creator");
		SETUTF8(0x25, "Producer");
		SETUTF8(0x26, "CreationDate");
		SETUTF8(0x27, "ModDate");
		return TRUE;

	case WM_COMMAND:
		EndDialog(hwnd, 1);
		return TRUE;
	}
	return FALSE;
}

void info()
{
	int code = DialogBoxW(NULL, L"IDD_DLOGINFO", hwndframe, dloginfoproc);
	if (code <= 0)
		winerror(&gapp, "cannot create info dialog");
}

INT_PTR CALLBACK
dlogaboutproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch(message)
	{
	case WM_INITDIALOG:
		SetDlgItemTextA(hwnd, 2, pdfapp_version(&gapp));
		SetDlgItemTextA(hwnd, 3, pdfapp_usage(&gapp));
		return TRUE;
	case WM_COMMAND:
		EndDialog(hwnd, 1);
		return TRUE;
	}
	return FALSE;
}

void winhelp(pdfapp_t*app)
{
	int code = DialogBoxW(NULL, L"IDD_DLOGABOUT", hwndframe, dlogaboutproc);
	if (code <= 0)
		winerror(&gapp, "cannot create help dialog");
}

/*
 * Main window
 */

void winopen()
{
	WNDCLASS wc;
	HMENU menu;
	RECT r;
	ATOM a;

	/* Create and register window frame class */
	memset(&wc, 0, sizeof(wc));
	wc.style = 0;
	wc.lpfnWndProc = frameproc;
	wc.cbClsExtra = 0;
	wc.cbWndExtra = 0;
	wc.hInstance = GetModuleHandle(NULL);
	wc.hIcon = LoadIconA(wc.hInstance, "IDI_ICONAPP");
	wc.hCursor = NULL; //LoadCursor(NULL, IDC_ARROW);
	wc.hbrBackground = NULL;
	wc.lpszMenuName = NULL;
	wc.lpszClassName = L"FrameWindow";
	a = RegisterClassW(&wc);
	if (!a)
		winerror(&gapp, "cannot register frame window class");

	/* Create and register window view class */
	memset(&wc, 0, sizeof(wc));
	wc.style = CS_HREDRAW | CS_VREDRAW;
	wc.lpfnWndProc = viewproc;
	wc.cbClsExtra = 0;
	wc.cbWndExtra = 0;
	wc.hInstance = GetModuleHandle(NULL);
	wc.hIcon = NULL;
	wc.hCursor = NULL;
	wc.hbrBackground = NULL;
	wc.lpszMenuName = NULL;
	wc.lpszClassName = L"ViewWindow";
	a = RegisterClassW(&wc);
	if (!a)
		winerror(&gapp, "cannot register view window class");

	/* Get screen size */
	SystemParametersInfo(SPI_GETWORKAREA, 0, &r, 0);
	gapp.scrw = r.right - r.left;
	gapp.scrh = r.bottom - r.top;

	/* Create cursors */
	arrowcurs = LoadCursor(NULL, IDC_ARROW);
	handcurs = LoadCursor(NULL, IDC_HAND);
	waitcurs = LoadCursor(NULL, IDC_WAIT);
	caretcurs = LoadCursor(NULL, IDC_IBEAM);

	/* And a background color */
	bgbrush = CreateSolidBrush(RGB(0x70,0x70,0x70));
	shbrush = CreateSolidBrush(RGB(0x40,0x40,0x40));

	/* Init DIB info for buffer */
	dibinf = malloc(sizeof(BITMAPINFO) + 12);
	assert(dibinf);
	dibinf->bmiHeader.biSize = sizeof(dibinf->bmiHeader);
	dibinf->bmiHeader.biPlanes = 1;
	dibinf->bmiHeader.biBitCount = 32;
	dibinf->bmiHeader.biCompression = BI_RGB;
	dibinf->bmiHeader.biXPelsPerMeter = 2834;
	dibinf->bmiHeader.biYPelsPerMeter = 2834;
	dibinf->bmiHeader.biClrUsed = 0;
	dibinf->bmiHeader.biClrImportant = 0;
	dibinf->bmiHeader.biClrUsed = 0;

	/* Create window */
	hwndframe = CreateWindowW(L"FrameWindow", // window class name
	NULL, // window caption
	WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
	CW_USEDEFAULT, CW_USEDEFAULT, // initial position
	CW_USEDEFAULT, CW_USEDEFAULT, // initial x size
	0, // parent window handle
	0, // window menu handle
	GetModuleHandle(NULL), // program instance handle
	0); // creation parameters
	if (!hwndframe)
		winerror(&gapp, "cannot create frame");

	hwndview = CreateWindowW(L"ViewWindow", // window class name
	NULL,
	WS_VISIBLE | WS_CHILD,
	CW_USEDEFAULT, CW_USEDEFAULT,
	CW_USEDEFAULT, CW_USEDEFAULT,
	hwndframe, 0, 0, 0);
	if (!hwndview)
		winerror(&gapp, "cannot create view");
	else
	{
		HMODULE hUser32 = GetModuleHandleA("user32.dll");
		BOOL (WINAPI *pRegisterTouchWindow)(HWND, ULONG) = (void*)GetProcAddress(hUser32, "RegisterTouchWindow");
		if (pRegisterTouchWindow)
			pRegisterTouchWindow(hwndview, TWF_WANTPALM);
	}

	hdc = NULL;

	SetWindowTextW(hwndframe, L"Majeh's PDF Viewer");

	menu = GetSystemMenu(hwndframe, 0);
	AppendMenuW(menu, MF_SEPARATOR, 0, NULL);
	AppendMenuW(menu, MF_STRING, ID_ABOUT, L"About Majeh's PDF Viewer...");
	AppendMenuW(menu, MF_STRING, ID_DOCINFO, L"Document Properties...");

	SetCursor(arrowcurs);
	}

	static void
	do_close(pdfapp_t *app)
	{
	pdfapp_close(app);
	free(dibinf);
	}

	void winclose(pdfapp_t *app)
	{
	if (pdfapp_preclose(app))
	{
		do_close(app);
		exit(0);
	}
	}

	void wincursor(pdfapp_t *app, int curs)
	{
	if (curs == ARROW)
		SetCursor(arrowcurs);
	if (curs == HAND)
		SetCursor(handcurs);
	if (curs == WAIT)
		SetCursor(waitcurs);
	if (curs == CARET)
		SetCursor(caretcurs);
	}

	void wintitle(pdfapp_t *app, char *title)
	{
	wchar_t wide[256], *dp;
	char *sp;
	int rune;

	dp = wide;
	sp = title;
	while (*sp && dp < wide + 255)
	{
		sp += fz_chartorune(&rune, sp);
		*dp++ = rune;
	}
	*dp = 0;

	SetWindowTextW(hwndframe, wide);
	}

	void windrawrect(pdfapp_t *app, int x0, int y0, int x1, int y1)
	{
	RECT r;
	r.left = x0;
	r.top = y0;
	r.right = x1;
	r.bottom = y1;
	FillRect(hdc, &r, (HBRUSH)GetStockObject(WHITE_BRUSH));
	}

	void windrawstring(pdfapp_t *app, int x, int y, char *s)
	{
	HFONT font = (HFONT)GetStockObject(ANSI_FIXED_FONT);
	SelectObject(hdc, font);
	TextOutA(hdc, x, y - 12, s, strlen(s));
	}

	void winblitsearch()
	{
	if (gapp.issearching)
	{
		char buf[sizeof(gapp.search) + 50];
		sprintf(buf, "Search: %s", gapp.search);
		windrawrect(&gapp, 0, 0, gapp.winw, 30);
		windrawstring(&gapp, 10, 20, buf);
	}
	}

	void winblit()
	{
	int image_w = fz_pixmap_width(gapp.ctx, gapp.image);
	int image_h = fz_pixmap_height(gapp.ctx, gapp.image);
	int image_n = fz_pixmap_components(gapp.ctx, gapp.image);
	unsigned char *samples = fz_pixmap_samples(gapp.ctx, gapp.image);
	int x0 = gapp.panx;
	int y0 = gapp.pany;
	int x1 = gapp.panx + image_w;
	int y1 = gapp.pany + image_h;
	RECT r;

	if (gapp.image)
	{
		if (gapp.iscopying || justcopied)
		{
			pdfapp_invert(&gapp, &gapp.selr);
			justcopied = 1;
		}

		pdfapp_inverthit(&gapp);

		dibinf->bmiHeader.biWidth = image_w;
		dibinf->bmiHeader.biHeight = -image_h;
		dibinf->bmiHeader.biSizeImage = image_h * 4;

		if (image_n == 2)
		{
			int i = image_w * image_h;
			unsigned char *color = malloc(i*4);
			unsigned char *s = samples;
			unsigned char *d = color;
			for (; i > 0 ; i--)
			{
				d[2] = d[1] = d[0] = *s++;
				d[3] = *s++;
				d += 4;
			}
			SetDIBitsToDevice(hdc,
				gapp.panx, gapp.pany, image_w, image_h,
				0, 0, 0, image_h, color,
				dibinf, DIB_RGB_COLORS);
			free(color);
		}
		if (image_n == 4)
		{
			SetDIBitsToDevice(hdc,
				gapp.panx, gapp.pany, image_w, image_h,
				0, 0, 0, image_h, samples,
				dibinf, DIB_RGB_COLORS);
		}

		pdfapp_inverthit(&gapp);

		if (gapp.iscopying || justcopied)
		{
			pdfapp_invert(&gapp, &gapp.selr);
			justcopied = 1;
		}
	}

	/* Grey background */
	r.top = 0; r.bottom = gapp.winh;
	r.left = 0; r.right = x0;
	FillRect(hdc, &r, bgbrush);
	r.left = x1; r.right = gapp.winw;
	FillRect(hdc, &r, bgbrush);
	r.left = 0; r.right = gapp.winw;
	r.top = 0; r.bottom = y0;
	FillRect(hdc, &r, bgbrush);
	r.top = y1; r.bottom = gapp.winh;
	FillRect(hdc, &r, bgbrush);

	/* Drop shadow */
	r.left = x0 + 2;
	r.right = x1 + 2;
	r.top = y1;
	r.bottom = y1 + 2;
	FillRect(hdc, &r, shbrush);
	r.left = x1; r.right = x1 + 2;
	r.top = y0 + 2;
	r.bottom = y1;
	FillRect(hdc, &r, shbrush);

	winblitsearch();
	}

	void winresize(pdfapp_t *app, int w, int h)
	{
	WINDOWPLACEMENT wp;
	wp.length = sizeof(wp);
	GetWindowPlacement(hwndframe, &wp);
	if (wp.showCmd == SW_SHOWMAXIMIZED)
	{
		wp.showCmd = SW_SHOWNORMAL;
		SetWindowPlacement(hwndframe, &wp);
	}
	ShowWindow(hwndframe, SW_SHOWNORMAL);
	w += GetSystemMetrics(SM_CXFRAME) * 2;
	h += GetSystemMetrics(SM_CYFRAME) * 2;
	h += GetSystemMetrics(SM_CYCAPTION);
	SetWindowPos(hwndframe, 0, 0, 0, w, h, SWP_NOZORDER | SWP_NOMOVE);
	}

void winrepaint(pdfapp_t *app)
{
	InvalidateRect(hwndview, NULL, 0);
}

void winrepaintsearch(pdfapp_t *app)
{
	// TODO: invalidate only search area and
	// call only search redraw routine.
	InvalidateRect(hwndview, NULL, 0);
}

void winfullscreen(pdfapp_t *app, int state)
{
	static WINDOWPLACEMENT savedplace;
	static int isfullscreen = 0;
	if (state && !isfullscreen)
	{
		GetWindowPlacement(hwndframe, &savedplace);
		SetWindowLong(hwndframe, GWL_STYLE, WS_POPUP | WS_VISIBLE);
		SetWindowPos(hwndframe, NULL, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED);
		ShowWindow(hwndframe, SW_SHOWMAXIMIZED);
		isfullscreen = 1;
	}
	if (!state && isfullscreen)
	{
		SetWindowLong(hwndframe, GWL_STYLE, WS_OVERLAPPEDWINDOW);
		SetWindowPos(hwndframe, NULL, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED);
		SetWindowPlacement(hwndframe, &savedplace);
		isfullscreen = 0;
	}
}

/*
 * Event handling
 */

void windocopy(pdfapp_t *app)
{
	HGLOBAL handle;
	unsigned short *ucsbuf;

	if (!OpenClipboard(hwndframe))
		return;
	EmptyClipboard();

	handle = GlobalAlloc(GMEM_MOVEABLE, 4096 * sizeof(unsigned short));
	if (!handle)
	{
		CloseClipboard();
		return;
	}

	ucsbuf = GlobalLock(handle);
	pdfapp_oncopy(&gapp, ucsbuf, 4096);
	GlobalUnlock(handle);

	SetClipboardData(CF_UNICODETEXT, handle);
	CloseClipboard();

	justcopied = 1;	/* keep inversion around for a while... */
}

void winreloadfile(pdfapp_t *app)
{
	pdfapp_close(app);
	pdfapp_open(app, filename, 1);
}

void winreloadpage(pdfapp_t *app)
{
	SendMessage(hwndview, WM_APP, 0, 0);
}

void winopenuri(pdfapp_t *app, char *buf)
{
	ShellExecuteA(hwndframe, "open", buf, 0, 0, SW_SHOWNORMAL);
}

#define OUR_TIMER_ID 1

void winadvancetimer(pdfapp_t *app, float delay)
{
	timer_pending = 1;
	SetTimer(hwndview, OUR_TIMER_ID, (unsigned int)(1000*delay), NULL);
}

static void killtimer(pdfapp_t *app)
{
	timer_pending = 0;
}

void handlekey(int c)
{
	if (timer_pending)
		killtimer(&gapp);

	if (GetCapture() == hwndview)
		return;

	if (justcopied)
	{
		justcopied = 0;
		winrepaint(&gapp);
	}

	/* translate VK into ASCII equivalents */
	if (c > 256)
	{
		switch (c - 256)
		{
		case VK_F1: c = '?'; break;
		case VK_ESCAPE: c = '\033'; break;
		case VK_DOWN: c = 'j'; break;
		case VK_UP: c = 'k'; break;
		case VK_LEFT: c = 'b'; break;
		case VK_RIGHT: c = ' '; break;
		case VK_PRIOR: c = ','; break;
		case VK_NEXT: c = '.'; break;
		}
	}

	pdfapp_onkey(&gapp, c);
	winrepaint(&gapp);
}

void handlemouse(int x, int y, int btn, int state)
{
	if (state != 0 && timer_pending)
		killtimer(&gapp);

	if (state != 0 && justcopied)
	{
		justcopied = 0;
		winrepaint(&gapp);
	}

	if (state == 1)
		SetCapture(hwndview);
	if (state == -1)
		ReleaseCapture();

	pdfapp_onmouse(&gapp, x, y, btn, 0, state);
}

LRESULT CALLBACK
frameproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	switch(message)
	{
	case WM_SETFOCUS:
		PostMessage(hwnd, WM_APP+5, 0, 0);
		return 0;
	case WM_APP+5:
		SetFocus(hwndview);
		return 0;

	case WM_DESTROY:
		PostQuitMessage(0);
		return 0;

	case WM_SYSCOMMAND:
		if (wParam == ID_ABOUT)
		{
			winhelp(&gapp);
			return 0;
		}
		if (wParam == ID_DOCINFO)
		{
			info();
			return 0;
		}
		break;

	case WM_SIZE:
	{
		// More generally, you should use GetEffectiveClientRect
		// if you have a toolbar etc.
		RECT rect;
		GetClientRect(hwnd, &rect);
		MoveWindow(hwndview, rect.left, rect.top,
		rect.right-rect.left, rect.bottom-rect.top, TRUE);
		if (wParam == SIZE_MAXIMIZED)
			gapp.shrinkwrap = 0;
		return 0;
	}

	case WM_SIZING:
		gapp.shrinkwrap = 0;
		break;

	case WM_NOTIFY:
	case WM_COMMAND:
		return SendMessage(hwndview, message, wParam, lParam);

	case WM_CLOSE:
		if (!pdfapp_preclose(&gapp))
			return 0;
	}

	return DefWindowProc(hwnd, message, wParam, lParam);
}

LRESULT CALLBACK
viewproc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
	static int oldx = 0;
	static int oldy = 0;
	int x = (signed short) LOWORD(lParam);
	int y = (signed short) HIWORD(lParam);

	/* Filter out touch-generated mouse messages for multi-finger gestures to prevent flicker.
	 * We allow single-finger touch to pass through as mouse events. */
	if (message >= WM_MOUSEFIRST && message <= WM_MOUSELAST)
	{
		if ((GetMessageExtraInfo() & 0xFFFFFF00) == 0xFF515700)
		{
			if (message == WM_RBUTTONDOWN || message == WM_RBUTTONUP || message == WM_MBUTTONDOWN || message == WM_MBUTTONUP)
				return 0;
			if (touch_max_active > 1)
				return 0;
		}
	}
	if (message == WM_CONTEXTMENU)
	{
		if ((GetMessageExtraInfo() & 0xFFFFFF00) == 0xFF515700)
			return 0;
	}

	switch (message)
	{
	case WM_SIZE:
		if (wParam == SIZE_MINIMIZED)
			return 0;
		if (wParam == SIZE_MAXIMIZED)
			gapp.shrinkwrap = 0;
		pdfapp_onresize(&gapp, LOWORD(lParam), HIWORD(lParam));
		break;

	case WM_PAINT:
	{
		PAINTSTRUCT ps;
		hdc = BeginPaint(hwnd, &ps);
		winblit();
		hdc = NULL;
		EndPaint(hwnd, &ps);
		pdfapp_postblit(&gapp);
		return 0;
	}

	case WM_ERASEBKGND:
		return 1;

	case WM_LBUTTONDOWN:
		SetFocus(hwndview);
		oldx = x; oldy = y;
		handlemouse(x, y, 1, 1);
		return 0;
	case WM_MBUTTONDOWN:
		SetFocus(hwndview);
		oldx = x; oldy = y;
		handlemouse(x, y, 2, 1);
		return 0;
	case WM_RBUTTONDOWN:
		SetFocus(hwndview);
		oldx = x; oldy = y;
		handlemouse(x, y, 3, 1);
		return 0;

	case WM_LBUTTONUP:
		oldx = x; oldy = y;
		handlemouse(x, y, 1, -1);
		return 0;
	case WM_MBUTTONUP:
		oldx = x; oldy = y;
		handlemouse(x, y, 2, -1);
		return 0;
	case WM_RBUTTONUP:
		oldx = x; oldy = y;
		handlemouse(x, y, 3, -1);
		return 0;

	case WM_MOUSEMOVE:
		oldx = x; oldy = y;
		handlemouse(x, y, 0, 0);
		return 0;

	/* Mouse wheel */

	case WM_MOUSEWHEEL:
		if ((signed short)HIWORD(wParam) > 0)
			handlekey(LOWORD(wParam) & MK_SHIFT ? '+' : 'k');
		else
			handlekey(LOWORD(wParam) & MK_SHIFT ? '-' : 'j');
		return 0;

	/* Timer */
	case WM_TIMER:
		if (wParam == OUR_TIMER_ID && timer_pending && gapp.presentation_mode)
		{
			timer_pending = 0;
			handlekey(VK_RIGHT + 256);
			handlemouse(oldx, oldy, 0, 0); /* update cursor */
			return 0;
		}
		break;

	/* Keyboard events */

	case WM_KEYDOWN:
		if (wParam == 'P' && (GetKeyState(VK_CONTROL) & 0x8000))
		{
			winprint(&gapp);
			return 0;
		}
		/* only handle special keys */
		switch (wParam)
		{
		case VK_F1:
		case VK_LEFT:
		case VK_UP:
		case VK_PRIOR:
		case VK_RIGHT:
		case VK_DOWN:
		case VK_NEXT:
		case VK_ESCAPE:
			handlekey(wParam + 256);
			handlemouse(oldx, oldy, 0, 0);	/* update cursor */
			return 0;
		}
		return 1;

	/* unicode encoded chars, including escape, backspace etc... */
	case WM_CHAR:
		if (wParam < 256)
		{
			handlekey(wParam);
			handlemouse(oldx, oldy, 0, 0);	/* update cursor */
		}
		return 0;

	/* We use WM_APP to trigger a reload and repaint of a page */
	case WM_APP:
		pdfapp_reloadpage(&gapp);
		break;

	case WM_TOUCH:
	{
		HMODULE hUser32 = GetModuleHandleA("user32.dll");
		BOOL (WINAPI *pGetTouchInputInfo)(HTOUCHINPUT, UINT, PTOUCHINPUT, int) = (void*)GetProcAddress(hUser32, "GetTouchInputInfo");
		BOOL (WINAPI *pCloseTouchInputHandle)(HTOUCHINPUT) = (void*)GetProcAddress(hUser32, "CloseTouchInputHandle");

		if (pGetTouchInputInfo && pCloseTouchInputHandle)
		{
			UINT cInputs = LOWORD(wParam);
			TOUCHINPUT *pInputs = malloc(sizeof(TOUCHINPUT) * cInputs);
			if (pInputs)
			{
				if (pGetTouchInputInfo((HTOUCHINPUT)lParam, cInputs, pInputs, sizeof(TOUCHINPUT)))
				{
					UINT i;
					for (i = 0; i < cInputs; i++)
					{
						TOUCHINPUT *ti = &pInputs[i];
						POINT pt;
						pt.x = TOUCH_COORD_TO_PIXEL(ti->x);
						pt.y = TOUCH_COORD_TO_PIXEL(ti->y);
						ScreenToClient(hwnd, &pt);

						int found = -1;
						int j;
						for (j = 0; j < 10; j++)
						{
							if (touch_pts[j].active && touch_pts[j].id == ti->dwID)
							{
								found = j;
								break;
							}
						}

						if (ti->dwFlags & TOUCHEVENTF_DOWN)
						{
							int already_active = 0;
							for (j = 0; j < 10; j++) if (touch_pts[j].active) already_active++;
							if (already_active == 0) {
								touch_max_active = 0;
								gesture_fired = 0;
							}
							for (j = 0; j < 10; j++)
							{
								if (!touch_pts[j].active)
								{
									touch_pts[j].id = ti->dwID;
									touch_pts[j].active = 1;
									touch_pts[j].start_x = pt.x;
									touch_pts[j].start_y = pt.y;
									touch_pts[j].x = pt.x;
									touch_pts[j].y = pt.y;
									if (already_active + 1 > touch_max_active)
										touch_max_active = already_active + 1;
									break;
								}
							}
						}
						else if (ti->dwFlags & TOUCHEVENTF_MOVE)
						{
							if (found != -1)
							{
								touch_pts[found].x = pt.x;
								touch_pts[found].y = pt.y;
							}
						}
						else if (ti->dwFlags & TOUCHEVENTF_UP)
						{
							if (found != -1)
								touch_pts[found].active = 0;
						}
					}

					int active_count = 0;
					int active_indices[10];
					long sum_dx = 0, sum_dy = 0;
					int k;
					for (k = 0; k < 10; k++)
					{
						if (touch_pts[k].active)
						{
							active_indices[active_count++] = k;
							sum_dx += (touch_pts[k].x - touch_pts[k].start_x);
							sum_dy += (touch_pts[k].y - touch_pts[k].start_y);
						}
					}

					if (active_count >= 2)
					{
						touch_point_t *p1 = &touch_pts[active_indices[0]];
						touch_point_t *p2 = &touch_pts[active_indices[1]];

						int dx1 = p1->x - p1->start_x;
						int dy1 = p1->y - p1->start_y;
						int dx2 = p2->x - p2->start_x;
						int dy2 = p2->y - p2->start_y;

						int curr_dx = p1->x - p2->x;
						int curr_dy = p1->y - p2->y;
						long curr_dist_sq = (long)curr_dx * curr_dx + (long)curr_dy * curr_dy;

						int init_dx = p1->start_x - p2->start_x;
						int init_dy = p1->start_y - p2->start_y;
						long init_dist_sq = (long)init_dx * init_dx + (long)init_dy * init_dy;

						/* Continuous Pinch Zoom (Ratio check) */
						if (init_dist_sq > 400 && (curr_dist_sq > init_dist_sq * 1.5 || curr_dist_sq < init_dist_sq * 0.6))
						{
							if (curr_dist_sq > init_dist_sq)
								handlekey('+');
							else
								handlekey('-');

							p1->start_x = p1->x; p1->start_y = p1->y;
							p2->start_x = p2->x; p2->start_y = p2->y;
							gesture_fired = 1;
						}
						/* 2-Finger Swipe (Stabilized) */
						else if (!gesture_fired && active_count == 2 && touch_max_active == 2)
						{
							if ((long)dx1*dx2 + (long)dy1*dy2 > 0) /* Move same direction */
							{
								int avg_dx = (dx1 + dx2) / 2;
								int avg_dy = (dy1 + dy2) / 2;
								if (avg_dx*avg_dx + avg_dy*avg_dy > 1600) /* 40px */
								{
									int key = 0;
									if (abs(avg_dx) > abs(avg_dy))
										key = (avg_dx > 0) ? 'i' : 'H';
									else
										key = (avg_dy > 0) ? 'f' : 'W';
									if (key) handlekey(key);
									gesture_fired = 1;
								}
							}
						}
						/* 3-Finger Swipe (Stabilized) */
						else if (!gesture_fired && active_count == 3 && touch_max_active == 3)
						{
							int avg_dx = sum_dx / 3;
							int avg_dy = sum_dy / 3;
							if (avg_dx*avg_dx + avg_dy*avg_dy > 1600) /* 40px */
							{
								int key = 0;
								if (abs(avg_dx) > abs(avg_dy))
									key = (avg_dx > 0) ? 'R' : 'L';
								else
									key = (avg_dy > 0) ? 'Z' : 'C';
								if (key) handlekey(key);
								gesture_fired = 1;
							}
						}
					}
				}
				free(pInputs);
			}
			pCloseTouchInputHandle((HTOUCHINPUT)lParam);
			return 0;
		}
		break;
	}

	case WM_GESTURE:
		return 0; /* Let WM_TOUCH handle it */

	}

	fflush(stdout);

	/* Pass on unhandled events to Windows */
	return DefWindowProc(hwnd, message, wParam, lParam);
}

int WINAPI
WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nShowCmd)
{
	int argc;
	LPWSTR *argv = CommandLineToArgvW(GetCommandLineW(), &argc);
	char argv0[256];
	MSG msg;
	int code;
	fz_context *ctx;
	int arg;
	int bps = 0;

	ctx = fz_new_context(NULL, NULL, FZ_STORE_DEFAULT);
	if (!ctx)
	{
		fprintf(stderr, "cannot initialise context\n");
		exit(1);
	}
	pdfapp_init(ctx, &gapp);

	GetModuleFileNameA(NULL, argv0, sizeof argv0);
	install_app(argv0);

	winopen();

	arg = 1;
	while (arg < argc)
	{
		if (!wcscmp(argv[arg], L"-p"))
		{
			if (arg+1 < argc)
				bps = _wtoi(argv[++arg]);
			else
				bps = 4096;
		}
		else
			break;
		arg++;
	}

	if (arg < argc)
	{
		wcscpy(wbuf, argv[arg]);
	}
	else
	{
		if (!winfilename(wbuf, nelem(wbuf)))
			exit(0);
	}

	code = WideCharToMultiByte(CP_UTF8, 0, wbuf, -1, filename, sizeof filename, NULL, NULL);
	if (code == 0)
		winerror(&gapp, "cannot convert filename to utf-8");

	if (bps)
		pdfapp_open_progressive(&gapp, filename, 0, bps);
	else
		pdfapp_open(&gapp, filename, 0);

	while (GetMessage(&msg, NULL, 0, 0))
	{
		TranslateMessage(&msg);
		DispatchMessage(&msg);
	}

	do_close(&gapp);
	fz_free_context(ctx);

	return 0;
}
