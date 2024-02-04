using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

var path = args[0];
var image = Image.FromFile(path);
var orig = new Bitmap(image);

var palette = new List<Color>();
for (var x = 0; x < orig.Width; x++) {
    for (var y = 0; y < orig.Height; y++) {
        var pixel = orig.GetPixel(x, y);
        if (pixel.A < 255) continue;
        if (!palette.Contains(pixel)) {
            palette.Add(pixel);
        }
    }
}

if (palette.Count > 255) {
    throw new Exception("Too many colors");
}

var @new = new Bitmap(orig.Width, orig.Height, PixelFormat.Format8bppIndexed);
var output = @new.LockBits(new Rectangle(0, 0, orig.Width, orig.Height), ImageLockMode.WriteOnly,
    PixelFormat.Format8bppIndexed);

for (var x = 0; x < orig.Width; x++) {
    for (var y = 0; y < orig.Height; y++) {
        var pixel = orig.GetPixel(x, y);
        var index = palette.IndexOf(pixel);
        if (index == -1) index = 255;
        var offset = y * output.Stride + x;
        Marshal.WriteByte(output.Scan0, offset, (byte)index);
    }
}

@new.UnlockBits(output);

var pal = @new.Palette;
for (var i = 0; i < palette.Count; i++) {
    pal.Entries[i] = palette[i];
}

pal.Entries[255] = Color.FromArgb(0, 0, 0, 255);
@new.Palette = pal;

@new.Save("./models_out/" + Path.GetFileNameWithoutExtension(path) + ".bmp",
    ImageFormat.Bmp);
