import os
import cv2
import numpy as np
from tkinter import Tk, filedialog, Button, Label, StringVar, OptionMenu, Scale, HORIZONTAL
from PIL import Image

selected_files = []
output_folder = ""

# ---------- PROCESSING ----------

def invert_image(img):
    return 255 - img

def stretch_channel(channel, low_perc=0.5, high_perc=99.5):
    low = np.percentile(channel, low_perc)
    high = np.percentile(channel, high_perc)

    if high - low < 1:
        return channel

    stretched = (channel - low) * (255.0 / (high - low))
    return np.clip(stretched, 0, 255).astype(np.uint8)

def correct_color(img):
    channels = cv2.split(img)
    return cv2.merge([stretch_channel(ch) for ch in channels])

def load_image(path):
    img = Image.open(path).convert("RGB")
    return np.array(img)

def process_image(path):
    img = load_image(path)
    img = invert_image(img)
    img = correct_color(img)
    return img

# ---------- FILE SELECTION ----------

def select_files():
    global selected_files
    selected_files = filedialog.askopenfilenames(
        filetypes=[("Images", "*.jpg *.jpeg *.png *.tif *.tiff")]
    )
    input_label.config(text=f"{len(selected_files)} images selected")

def select_output():
    global output_folder
    output_folder = filedialog.askdirectory()
    output_label.config(text=f"Output: {output_folder}")

# ---------- SAVE FUNCTION ----------

def save_image(img_array, path, format_choice, quality):
    img = Image.fromarray(img_array)

    if format_choice == "JPG":
        img.save(path, "JPEG", quality=quality, subsampling=0)

    elif format_choice == "PNG":
        # PNG uses compression level (0 = best quality, 9 = most compression)
        compression = int((100 - quality) / 10)
        img.save(path, "PNG", compress_level=compression)

    elif format_choice == "TIFF":
        img.save(path, "TIFF", compression="none")

# ---------- PROCESS ----------

def process_batch():
    if not selected_files:
        status_label.config(text="No images selected")
        return

    if not output_folder:
        status_label.config(text="No output folder selected")
        return

    format_choice = format_var.get()
    quality = quality_slider.get()

    total = len(selected_files)
    success_count = 0

    for i, path in enumerate(selected_files):
        try:
            result = process_image(path)

            filename = os.path.basename(path)
            name, _ = os.path.splitext(filename)

            extension = format_choice.lower()
            out_path = os.path.join(output_folder, f"{name}_positive.{extension}")

            save_image(result, out_path, format_choice, quality)

            if os.path.exists(out_path):
                success_count += 1
            else:
                raise Exception("File not saved")

            status_label.config(text=f"Processing {i+1}/{total}")
            root.update_idletasks()

        except Exception as e:
            status_label.config(text=f"Error: {str(e)}")
            return

    status_label.config(text=f"Done! Saved {success_count}/{total} images")

# ---------- UI ----------

root = Tk()
root.title("Film Negative Converter v2")

input_label = Label(root, text="No images selected")
input_label.pack()

Button(root, text="Select Images", command=select_files).pack()

output_label = Label(root, text="No output folder selected")
output_label.pack()

Button(root, text="Select Output Folder", command=select_output).pack()

# Format selection
format_var = StringVar(root)
format_var.set("JPG")

Label(root, text="Output Format").pack()
OptionMenu(root, format_var, "JPG", "PNG", "TIFF").pack()

# Quality slider
Label(root, text="Quality (for JPG/PNG)").pack()
quality_slider = Scale(root, from_=50, to=100, orient=HORIZONTAL)
quality_slider.set(95)
quality_slider.pack()

Button(root, text="Process Images", command=process_batch).pack()

status_label = Label(root, text="")
status_label.pack()

root.mainloop()