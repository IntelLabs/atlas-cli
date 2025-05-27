import argparse
import json
import logging
from pathlib import Path
from typing import Dict, Tuple
from pathlib import Path
from typing import Dict, Tuple

import torch.utils.data as data
from torchvision import transforms as tfs
from torchvision.datasets import MNIST

import torch
import torch.nn as nn
import torch.utils.data as data
from tqdm import tqdm


def load_mnist(
    path_to_data: Path, batch_size: int
) -> Tuple[data.DataLoader, data.DataLoader]:
    mean = [0.5]
    std = [0.5]
    transforms = tfs.Compose([tfs.ToTensor(), tfs.Normalize(mean, std)])

    mnist_train = MNIST(path_to_data, train=True, transform=transforms, download=False)
    mnist_test = MNIST(path_to_data, train=False, transform=transforms, download=False)

    train_loader = data.DataLoader(mnist_train, batch_size=batch_size, shuffle=True)
    test_loader = data.DataLoader(mnist_test, batch_size=batch_size)

    return train_loader, test_loader

class MNIST_CNN(nn.Module):
    def __init__(self):
        super(MNIST_CNN, self).__init__()

        self.conv1 = nn.Conv2d(1, 32, kernel_size=5)
        self.conv2 = nn.Conv2d(32, 64, kernel_size=5)

        self.relu = nn.ReLU(True)
        self.pool = nn.MaxPool2d(kernel_size=2)
        self.fc1 = nn.Linear(64 * 4 * 4, 10)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        x = self.relu(self.pool(self.conv1(x)))
        x = self.relu(self.pool(self.conv2(x)))
        x = x.view(-1, 64 * 4 * 4)
        x = self.fc1(x)

        return nn.functional.log_softmax(x, dim=1)


def train(
    model: nn.Module,
    data_loader: data.DataLoader,
    conf: Dict,
    device: torch.device,
    log: logging.Logger,
) -> nn.Module:
    log.info("Training...")

    model.to(device)
    model.train()
    optimiser = torch.optim.SGD(model.parameters(), lr=conf["lr"])
    loss_func = nn.CrossEntropyLoss()

    for epoch in range(conf["epochs"]):
        for _, (batch_x, batch_y) in enumerate(
            tqdm(
                data_loader,
                unit="batches",
                desc=f"Training epoch {epoch+1}/{conf['epochs']}",
            )
        ):
            batch_x = batch_x.to(device)
            batch_y = batch_y.to(device)

            optimiser.zero_grad()

            ypred = model(batch_x)
            loss = loss_func(input=ypred, target=batch_y)
            loss.backward()
            optimiser.step()

    model.eval()
    return model


def save_model_and_conf(
    model: nn.Module, conf: Dict, output_dir: Path
) -> Tuple[Path, Path]:
    model_path = output_dir / "model.pkl"
    torch.save(model.state_dict(), model_path)
    if not model_path.exists():
        raise IOError("failed to save the model")

    conf_path = output_dir / "training_conf.json"
    with open(conf_path, "w", encoding="utf-8") as f:
        json.dump(conf, f, ensure_ascii=False, indent=4)

    if not conf_path.exists():
        raise IOError("failed to save the config")

    return model_path, conf_path


def create_dir_if_doesnt_exist(path_to_dir: Path):
    path = Path(path_to_dir)
    if not path.exists():
        log.warning(f"{path_to_dir} does not exist. Creating...")
        path.mkdir(parents=True, exist_ok=True)


if __name__ == "__main__":
    logging.basicConfig(level=logging.NOTSET)
    log = logging.getLogger("TRAIN")

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path_to_data",
        type=str,
        default="./output/data",
        help="Path to store/load the data",
    )
    parser.add_argument(
        "--path_to_output",
        type=str,
        default="./output/train",
        help="Path to save output",
    )
    parser.add_argument(
        "--batch_size", type=int, default=128, help="Training batch size"
    )
    parser.add_argument("--lr", type=float, default=0.5, help="Training batch size")
    parser.add_argument("--epochs", type=int, default=1, help="Training epochs")
    parser.add_argument(
        "--use_cuda", type=bool, default=False, help="Use CUDA for training"
    )

    args = parser.parse_args()
    conf = vars(args)

    if conf["use_cuda"]:
        if not torch.cuda.is_available():
            log.warning("use_cuda==True but no cuda devices found; will run on CPU")
            device = torch.device("cpu")
        else:
            device = torch.device("cuda")
    else:
        device = torch.device("cpu")

    train_loader, _ = load_mnist(
        path_to_data=conf["path_to_data"], batch_size=conf["batch_size"]
    )

    model = MNIST_CNN()

    trained_model = train(
        model=model, data_loader=train_loader, conf=conf, device=device, log=log
    )

    create_dir_if_doesnt_exist(Path(conf["path_to_output"]).resolve())
    model_path, conf_path = save_model_and_conf(
        model=trained_model,
        conf=conf,
        output_dir=Path(conf["path_to_output"]).resolve(),
    )
    log.info(f"Model saved in {model_path}")
    log.info(f"Config saved in {conf_path}")
