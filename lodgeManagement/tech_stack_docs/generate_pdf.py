import os
from fpdf import FPDF
import re

class PDF(FPDF):
    def header(self):
        self.set_font("helvetica", "B", 12)
        self.cell(0, 10, "Lodge Management System Documentation", border=False, ln=True, align="C")

    def footer(self):
        self.set_y(-15)
        self.set_font("helvetica", "I", 8)
        self.cell(0, 10, f"Page {self.page_no()}", align="C")

    def chapter_title(self, title):
        self.set_font("helvetica", "B", 16)
        self.cell(0, 10, title, ln=True, fill=False)
        self.ln(4)

    def chapter_body(self, body):
        self.set_font("helvetica", "", 12)
        # Basic markdown to text cleanup
        body = re.sub(r'#+\s', '', body)
        body = re.sub(r'\*\*(.*?)\*\*', r'\1', body)
        self.multi_cell(0, 7, body)
        self.ln()

def generate_pdf():
    pdf = PDF()
    pdf.add_page()
    
    docs_dir = "/workspaces/projects/lodgeManagement/tech_stack_docs"
    
    # Process modules
    modules_dir = os.path.join(docs_dir, "modules")
    for filename in sorted(os.listdir(modules_dir)):
        if filename.endswith(".md"):
            with open(os.path.join(modules_dir, filename), "r") as f:
                content = f.read()
                pdf.chapter_title(filename.replace(".md", "").replace("_", " ").title())
                pdf.chapter_body(content)

    # Process tutorials
    tutorials_dir = os.path.join(docs_dir, "tutorials")
    for filename in sorted(os.listdir(tutorials_dir)):
        if filename.endswith(".md"):
            with open(os.path.join(tutorials_dir, filename), "r") as f:
                content = f.read()
                pdf.chapter_title(filename.replace(".md", "").replace("_", " ").title())
                pdf.chapter_body(content)
                
    pdf.output("Lodge_Management_Docs.pdf")
    print("PDF generated: Lodge_Management_Docs.pdf")

if __name__ == "__main__":
    generate_pdf()
