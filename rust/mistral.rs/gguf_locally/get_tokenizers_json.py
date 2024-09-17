from transformers import AutoTokenizer

# model = input("microsoft/Orca-2-13b")
# model = input("elyza/ELYZA-japanese-Llama-2-7b-instruct")
model = "elyza/ELYZA-japanese-Llama-2-7b-instruct"
tok = AutoTokenizer.from_pretrained(model)
tok.save_pretrained("./")