param([switch]$Debug)

$Config = if ($Debug) { "debug" } else { "release" }
$MakeConfig = if ($Debug) { "build=debug" } else { "build=release" }

$gitPath = "C:\Program Files\Git\bin;C:\Program Files\Git\usr\bin"
$env:PATH = "$gitPath;$env:PATH"

Write-Host "--- Building in $Config mode ---"

Write-Host "--- Generating Headers ---"
if (!(Test-Path "generated")) { New-Item -ItemType Directory "generated" }
mingw32-make OS=MINGW CC=gcc $MakeConfig generate

Write-Host "--- Creating Build Folders ---"
$dirs = @("fitz", "pdf", "pdf/js", "xps", "cbz", "img", "tiff", "tools", "platform/x11", "freetype", "jbig2dec", "jpeg", "openjpeg", "zlib", "mujs")
foreach ($d in $dirs) { New-Item -ItemType Directory -Force -Path "build/$Config/$d" | Out-Null }

Write-Host "--- Compiling Libraries ---"
mingw32-make OS=MINGW CC=gcc $MakeConfig

Write-Host "--- Compiling Majeh's PDF Viewer GUI ($Config) ---"
windres platform/x11/win_res.rc -O coff -o build/$Config/win_res.obj

if ($Debug) {
    # Added -mwindows to suppress terminal
    gcc -Iinclude -Igenerated -Wall -pipe -g -DDEBUG -o build/$Config/majehs-viewer.exe platform/x11/win_main.c platform/x11/pdfapp.c build/$Config/win_res.obj build/$Config/libmajehpdfviewer.a build/$Config/libfreetype.a build/$Config/libjbig2dec.a build/$Config/libjpeg.a build/$Config/libopenjpeg.a build/$Config/libz.a build/$Config/libmujs.a -lgdi32 -lcomdlg32 -lcomctl32 -lwinspool -lm -mwindows
} else {
    # Added -mwindows to suppress terminal
    gcc -Iinclude -Igenerated -Wall -pipe -O1 -DNDEBUG -o build/$Config/majehs-viewer.exe platform/x11/win_main.c platform/x11/pdfapp.c build/$Config/win_res.obj build/$Config/libmajehpdfviewer.a build/$Config/libfreetype.a build/$Config/libjbig2dec.a build/$Config/libjpeg.a build/$Config/libopenjpeg.a build/$Config/libz.a build/$Config/libmujs.a -lgdi32 -lcomdlg32 -lcomctl32 -lwinspool -lm -mwindows
    Write-Host "--- Stripping symbols for smaller binary ---"
    strip build/release/majehs-viewer.exe
}

Write-Host "Build Complete: build/$Config/majehs-viewer.exe"
