#%%
import pyperclip
import fire

def output_clip():
    return pyperclip.paste()

if __name__ == '__main__':
    fire.Fire(output_clip)

# %%
